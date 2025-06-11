use std::io;
use std::io::Write;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use std::path::PathBuf;

use clap::Args;
use clap::{command, Parser};
use color_eyre::eyre;
use const_format::formatcp;
use eyre::eyre;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

use acir::{native_types::WitnessStack, FieldElement};
use acvm::BlackBoxFunctionSolver;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use brillig::MemoryAddress;
use brillig::Opcode as BrilligOpcode;
use brillig::{BinaryFieldOp, BinaryIntOp};
use nargo::foreign_calls::{
    layers, transcript::ReplayForeignCallExecutor, DefaultForeignCallBuilder,
};
use nargo::{foreign_calls::ForeignCallExecutor, NargoError};
use noir_artifact_cli::execution;
use noir_artifact_cli::execution::{ExecutionResults, ReturnValues};
use noir_artifact_cli::fs::{inputs::read_inputs_from_file, witness::save_witness_to_dir};
use noir_artifact_cli::{errors::CliError, Artifact};
use noirc_abi::input_parser::InputValue;
use noirc_abi::InputMap;
use noirc_driver::CompiledProgram;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
static VERSION_STRING: &str = formatcp!("version = {}\n", PKG_VERSION,);

/// Executes the given compiled circuit with specified inputs and solvers, returning output witnesses.
pub fn run_circuit_and_get_witnesses<B, E>(
    circuit: &CompiledProgram,
    blackbox_solver: &B,
    foreign_call_executor: &mut E,
    input_map: &InputMap,
    expected_return: Option<InputValue>,
) -> Result<ExecutionResults, CliError>
where
    B: BlackBoxFunctionSolver<FieldElement>,
    E: ForeignCallExecutor<FieldElement>,
{
    let initial_witness = circuit.abi.encode(&input_map, None)?;

    let witness_stack = nargo::ops::execute_program(
        &circuit.program,
        initial_witness,
        blackbox_solver,
        foreign_call_executor,
    )?;

    let main_witness = &witness_stack
        .peek()
        .expect("Should have at least one witness on the stack")
        .witness;

    let (_, actual_return) = circuit.abi.decode(main_witness)?;

    Ok(ExecutionResults {
        witness_stack,
        return_values: ReturnValues {
            actual_return,
            expected_return,
        },
    })
}

/// Executes the circuit with potentially replayed oracle transcript and returns the results.
fn execute(
    circuit: &CompiledProgram,
    args: &ExecuteCommand,
    input_map: &InputMap,
    expected_return: Option<InputValue>,
) -> Result<ExecutionResults, CliError> {
    // Build a custom foreign call executor that replays the Oracle transcript,
    // and use it as a base for the default executor. Using it as the innermost rather
    // than top layer so that any extra `print` added for debugging is handled by the
    // default, rather than trying to match it to the transcript.
    let transcript_executor = match args.oracle_file {
        Some(ref path) => layers::Either::Left(ReplayForeignCallExecutor::from_file(path)?),
        None => layers::Either::Right(layers::Empty),
    };

    let mut foreign_call_executor = DefaultForeignCallBuilder {
        output: std::io::stdout(),
        enable_mocks: false,
        resolver_url: args.oracle_resolver.clone(),
        root_path: None,
        package_name: None,
    }
    .build_with_base(transcript_executor);

    let blackbox_solver = Bn254BlackBoxSolver(args.pedantic_solving);

    run_circuit_and_get_witnesses(
        circuit,
        &blackbox_solver,
        &mut foreign_call_executor,
        input_map,
        expected_return,
    )
}

pub fn draw_random_constant<F>(
    destination: MemoryAddress,
    source: MemoryAddress,
    rng: &mut StdRng,
) -> BrilligOpcode<F> {
    std::env::set_var("ZKFUZZ_NOIR_SEED", format!("{}", rng.random::<u64>()));
    match source {
        MemoryAddress::Direct(address) => BrilligOpcode::Mov {
            destination,
            source: MemoryAddress::Direct(usize::MAX - address),
        },
        MemoryAddress::Relative(offset) => BrilligOpcode::Mov {
            destination,
            source: MemoryAddress::Relative(usize::MAX - offset),
        },
    }
}

/// Fuzzes a Noir program by mutating inputs and unconstrained functions, detecting under-constrained bugs.
///
/// Compares return values between original and mutated programs to identify behavioral divergence.
///
/// # Arguments
/// * `args` - Command-line execution arguments.
/// * `num_generation` - Number of fuzzing iterations to run.
/// * `rng` - Mutable random number generator seeded externally.
///
/// # Returns
/// Returns `Ok(())` if successful or reports early when a mismatch is found.
pub fn zkfuzz_run(
    args: ExecuteCommand,
    num_generation: usize,
    rng: &mut StdRng,
) -> Result<(), CliError> {
    let artifact = Artifact::read_from_file(&args.artifact_path)?;
    let artifact_name = args
        .artifact_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();

    if let Artifact::Program(program) = artifact {
        let mut circuit: CompiledProgram = program.into();

        let (initial_input_map, _initial_expected_return) =
            read_inputs_from_file(&args.prover_file, &circuit.abi)?;
        let mut input_map_keys: Vec<_> = initial_input_map.keys().collect();
        input_map_keys.sort();

        let original_unconstrained_functions = circuit.program.unconstrained_functions.clone();

        for i in 0..num_generation {
            print!("\r\x1b[2K🧬 Generation: {}/{}", i, num_generation);

            // ------------ Generating random inputs ------------------------ //
            let mut mutated_input_map = initial_input_map.clone();
            //let name = input_map_keys[rng.gen_range(0..input_map_keys.len())];
            for name in &input_map_keys {
                if let Some(InputValue::Field(_)) = mutated_input_map.get(name.clone()) {
                    mutated_input_map.insert(
                        name.clone().clone(),
                        InputValue::Field(FieldElement::from(rng.gen_range(0..2) as u32)),
                    );
                }
            }

            // ------------ Executing the original circuit ------------------ //
            circuit.program.unconstrained_functions = original_unconstrained_functions.clone();
            let original_result = match execute(&circuit, &args, &mutated_input_map, None) {
                Ok(results) => results.return_values.actual_return,
                Err(e) => {
                    if let CliError::CircuitExecutionError(ref err) = e {
                        execution::show_diagnostic(&circuit, err);
                    }
                    None
                }
            };

            // ------------ Mutating unconstrained functions ---------------- //

            let mut mutated_unconstrained_functions = original_unconstrained_functions.clone();

            let func_idx: usize = rng.gen_range(0..original_unconstrained_functions.len());
            let instr_pos: usize =
                rng.gen_range(0..original_unconstrained_functions[func_idx].bytecode.len());

            match mutated_unconstrained_functions[func_idx].bytecode[instr_pos] {
                BrilligOpcode::Mov {
                    destination,
                    source,
                } => {
                    mutated_unconstrained_functions[func_idx].bytecode[instr_pos] =
                        draw_random_constant(destination, source, rng);
                }
                /*
                BrilligOpcode::BinaryFieldOp {
                    destination,
                    op: _,
                    lhs,
                    rhs,
                } => {
                    mutated_unconstrained_functions[func_idx].bytecode[instr_pos] =
                        draw_random_constant(destination, rng);
                }
                BrilligOpcode::BinaryIntOp {
                    destination,
                    op: _,
                    lhs,
                    rhs: _,
                    bit_size,
                } => {
                    mutated_unconstrained_functions[func_idx].bytecode[instr_pos] =
                        draw_random_constant(destination, rng);
                }*/
                _ => {}
            }
            circuit.program.unconstrained_functions = mutated_unconstrained_functions;

            // ----------- Executing the mutated circuit -------------------- //
            let raw_mutated_result = catch_unwind(AssertUnwindSafe(|| {
                execute(&circuit, &args, &mutated_input_map, None)
            }));
            let mutated_result = match &raw_mutated_result {
                Ok(Ok(results)) => results.return_values.actual_return.clone(),
                Ok(Err(e)) => {
                    if let CliError::CircuitExecutionError(ref err) = e {
                        execution::show_diagnostic(&circuit, err);
                    }
                    None
                }
                Err(e) => {
                    eprintln!("{e:?}");
                    None
                }
            };

            match (&original_result, &mutated_result) {
                (Some(v), Some(u)) => {
                    if v != u {
                        print!("\r\x1b[2K🧬 Generation: {}/{}", i, num_generation);
                        io::stdout().flush().unwrap();
                        println!("  Under-Constrained");
                        println!("      Original Return Value: {:?}", v);
                        println!("       Mutated Return Value: {:?}", u);
                        return Ok(());
                    }
                }
                (_, _) => {}
            }
        }
    }
    println!("\nDone");

    Ok(())
}

fn parse_and_normalize_path(path: &str) -> eyre::Result<PathBuf> {
    use fm::NormalizePath;
    let mut path: PathBuf = path
        .parse()
        .map_err(|e| eyre!("failed to parse path: {e}"))?;
    if !path.is_absolute() {
        path = std::env::current_dir().unwrap().join(path).normalize();
    }
    Ok(path)
}

/// Execute a binary program or a circuit artifact.
#[derive(Debug, Clone, Args)]
pub struct ExecuteCommand {
    /// Path to the JSON build artifact (either a program or a contract).
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    pub artifact_path: PathBuf,

    /// Path to the Prover.toml file which contains the inputs and the
    /// optional return value in ABI format.
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    pub prover_file: PathBuf,

    /// Seed
    #[clap(long, short)]
    pub seed: Option<usize>,

    /// Path to the directory where the output witness should be saved.
    /// If empty then the results are discarded.
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    pub output_dir: Option<PathBuf>,

    /// Write the execution witness to named file
    ///
    /// Defaults to the name of the circuit being executed.
    #[clap(long, short)]
    pub witness_name: Option<String>,

    /// Name of the function to execute, if the artifact is a contract.
    #[clap(long)]
    pub contract_fn: Option<String>,

    /// Path to the oracle transcript that is to be replayed during the
    /// execution in response to foreign calls. The format is expected
    /// to be JSON Lines, with each request/response on a separate line.
    ///
    /// Note that a transcript might be invalid if the inputs change and
    /// the circuit takes a different path during execution.
    #[clap(long, conflicts_with = "oracle_resolver")]
    pub oracle_file: Option<PathBuf>,

    /// JSON RPC url to solve oracle calls.
    #[clap(long, conflicts_with = "oracle_file")]
    pub oracle_resolver: Option<String>,

    /// Root directory for the RPC oracle resolver.
    #[clap(long, value_parser = parse_and_normalize_path)]
    pub oracle_root_dir: Option<PathBuf>,

    /// Package name for the RPC oracle resolver
    #[clap(long)]
    pub oracle_package_name: Option<String>,

    /// Use pedantic ACVM solving, i.e. double-check some black-box function assumptions when solving.
    #[clap(long, default_value_t = false)]
    pub pedantic_solving: bool,
}

#[derive(Parser, Debug)]
#[command(name="noir-execute", author, version=VERSION_STRING, about, long_about = None)]
struct AExecutorCli {
    #[command(flatten)]
    command: ExecuteCommand,
}

pub fn start_cli() -> eyre::Result<()> {
    let AExecutorCli { command } = AExecutorCli::parse();
    let mut rng = StdRng::seed_from_u64(
        command
            .seed
            .unwrap_or_default()
            .try_into()
            .unwrap_or_default(),
    );
    zkfuzz_run(command, 1000, &mut rng);

    Ok(())
}

fn main() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::ACTIVE)
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .with_env_filter(EnvFilter::from_env("NOIR_LOG"))
        .init();

    if let Err(e) = start_cli() {
        eprintln!("{e:?}");
        std::process::exit(1);
    }
}
