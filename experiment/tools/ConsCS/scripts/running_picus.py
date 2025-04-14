import os
import subprocess
from datetime import datetime

def run_docker_command(solver, timeout, log_path, target_folder):
    os.makedirs(log_path, exist_ok=True)
    
    for file_path in os.listdir(target_folder):
        if file_path.endswith('.r1cs'):
            file_name = os.path.basename(file_path)
            base_name = file_name.split('.')[0]
            full_path = os.path.join(target_folder, file_path)
            log_file_path = os.path.join(log_path, f'{base_name}.log')

            print(f"=================== checking: {file_name} ===================")
            start_time = datetime.utcnow()
            print(f"====   start: {start_time}")

            command = [
                'docker', 'exec', 'ffa0f4c0ba2b', 'racket', './picus-dpvl-uniqueness.rkt',
                '--solver', solver, '--timeout', '5000', '--weak', '--r1cs', full_path
            ]

            start = datetime.now()

            with open(log_file_path, 'w') as log_file:
                process = subprocess.Popen(command, stdout=log_file, stderr=log_file)
                try:
                    process.wait(timeout=timeout)
                except subprocess.TimeoutExpired:
                    process.kill()
                    print(f"Timeout expired after {timeout} seconds")

            end_time = datetime.utcnow()
            elapsed_time = (datetime.now() - start).total_seconds()
            
            print(f"====     end: {end_time}")
            print(f"==== elapsed: {elapsed_time} seconds")

        
if __name__ == '__main__':
    import sys
    
    for type_folder in ["utils", "core"]:
        solver = "cvc5"
        timeout = 600
        log_path = os.path.join("./logs/", type_folder)
        target_folder = os.path.join("./benchmarks/", type_folder)
        run_docker_command(solver, timeout, log_path, target_folder)