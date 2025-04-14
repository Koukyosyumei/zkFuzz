import os
import subprocess
import time
from misc import *

def run_and_log_and_solve_one_path(target_folder, file_path, log_path, log_path_with_contributions, flag_str, cur_max_depth):  
    if not file_path.endswith('.r1cs'):
        return
    assert len(flag_str) == 3
    assert isinstance(cur_max_depth, str)
    file_name = os.path.basename(file_path)
    base_name = file_name.split('.')[0]
    full_path = os.path.join(target_folder, file_path)
    target_file_path = full_path
    print(target_file_path)
    if os.path.exists(log_path):
        existing_log_contents = read_from_file(log_path)
        if f"** filename: {base_name}.circom" in existing_log_contents:
            assert f"** filename: {base_name}.circom" in read_from_file(log_path_with_contributions)
            return

    command = ["python3", "analyze_circuit.py", target_file_path, log_path, log_path_with_contributions, flag_str, cur_max_depth]
    try:
        result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    except Exception as e:
        log_message = ""
        log_message += f"** filename: {base_name}.circom \n"
        log_message += f"** result: {message} \n"
        log_message += f"** time: {time_taken} \n"
        append_log(log_message, log_path)
        
        
    if result.stdout:
        print("Output:", result.stdout)
    if result.stderr:
        print("Error:", result.stderr)


# ablation, including main
for SIMPLIFICATION_flag in ["1", "0"]:
    for BPG_flag in ["1", "0"]:
        for ASSUMPTION_flag in ["1", "0"]:
            flag_str = SIMPLIFICATION_flag + BPG_flag + ASSUMPTION_flag
            cur_max_depth = '4'
            print(flag_str, cur_max_depth)
            
            log_path = f"./logs/our_logs_{flag_str}_{cur_max_depth}.log"
            log_path_with_contributions = f"./logs/our_logs_with_contributions_{flag_str}_{cur_max_depth}.log"
            for type_folder in ["utils", "core"]:
                target_folder = os.path.join("./picus_bench/", type_folder)
                for file_path in os.listdir(target_folder):
                    run_and_log_and_solve_one_path(target_folder, file_path, log_path, log_path_with_contributions, flag_str, cur_max_depth)

                    
                    
# testing depth                 
for cur_max_depth in range(10):
    cur_max_depth = str(cur_max_depth)
    if cur_max_depth == '4':
        continue
    flag_str = '111'
    
    print(flag_str, cur_max_depth)

    log_path = f"./logs/our_logs_{flag_str}_{cur_max_depth}.log"
    log_path_with_contributions = f"./logs/our_logs_with_contributions_{flag_str}_{cur_max_depth}.log"
    for type_folder in ["utils", "core"]:
        target_folder = os.path.join("./picus_bench/", type_folder)
        for file_path in os.listdir(target_folder):
            run_and_log_and_solve_one_path(target_folder, file_path, log_path, log_path_with_contributions, flag_str, cur_max_depth)

    
    