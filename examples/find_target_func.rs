use anyhow::Result;
use wasmparser::{Parser, Payload};

fn find_target_function_index(
    wasm_bytes: &[u8],
    target_function_name: String,
) -> Result<Option<u32>> {
    let parser = Parser::new(0);
    let mut function_index = 0u32;
    let mut export_section_found = false;

    for payload in parser.parse_all(wasm_bytes) {
        match payload? {
            Payload::ExportSection(reader) => {
                export_section_found = true;
                for export in reader {
                    let export = export?;
                    if export.name == target_function_name {
                        if let wasmparser::ExternalKind::Func = export.kind {
                            return Ok(Some(export.index));
                        }
                    }
                }
            }
            Payload::FunctionSection(reader) => {
                if !export_section_found {
                    function_index += reader.count();
                }
            }
            _ => {}
        }
    }

    Ok(None)
}

fn find_function_body_range(
    wasm_bytes: &[u8],
    target_func_index: u32,
) -> Option<std::ops::Range<usize>> {
    let parser = Parser::new(0);
    let mut current_func_index = 0u32;

    for payload in parser.parse_all(wasm_bytes) {
        if let Ok(Payload::CodeSectionEntry(reader)) = payload {
            if current_func_index == target_func_index {
                return Some(reader.range().start..reader.range().end);
            }
            current_func_index += 1;
        }
    }
    None
}

fn main() -> Result<()> {
    //  Magic Number and Version
    //      0, 97, 115, 109, 1, 0, 0, 0,
    //  Type Section
    //      1, 7, 1, 96, 2, 127, 127, 1, 127,
    //  Function Section
    //      3, 3, 2, 0, 0,
    //  Export Section
    //      7, 13, 2, 3, 109, 117, 108, 0, 0, 3, 97, 100, 100, 0, 1,
    //  Code Section
    //      10, 17, 2,
    //      7, 0, 32, 0, 32, 1, 108, 11, -- Function 0 (mul)
    //      7, 0, 32, 0, 32, 1, 106, 11, -- Function 1 (add)
    //  Custom Section
    //      0, 18, 4, 110, 97, 109, 101, 1, 11, 2, 0, 3, 109, 117, 108, 1, 3, 97, 100, 100

    let test_wasm = wat::parse_str(
        r#"
            (module
                (func $mul (export "mul") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.mul
                )
                (func $add (export "add") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add
                )
            )
        "#,
    )
    .unwrap();

    let target_func_index = find_target_function_index(&test_wasm, "add".to_string())?;
    let target_func_range = find_function_body_range(&test_wasm, target_func_index.unwrap());

    assert_eq!(target_func_index.unwrap(), 1);
    assert_eq!(target_func_range.clone().unwrap().start, 49);
    assert_eq!(target_func_range.clone().unwrap().end, 56);

    Ok(())
}
