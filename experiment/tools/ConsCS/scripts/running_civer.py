import os
import subprocess
import time

def run_circom_commands(directory, log_file_path):
    files = [f for f in os.listdir(directory) if f.endswith('.circom')]
    with open(log_file_path, 'w') as log_file:
        for file_name in files:
            full_path = os.path.join(directory, file_name)
            command = f"circom {full_path} --civer tags_specifications.circom --check_safety"
            log_file.write(f"Running command: {command}\n")
            start_time = time.time()
            process = subprocess.Popen(command, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            stdout, stderr = process.communicate()
            end_time = time.time()
            elapsed_time = end_time - start_time
            log_file.write("Standard Output:\n" + stdout.decode() + "\n")
            log_file.write("Standard Error:\n" + stderr.decode() + "\n")
            log_file.write(f"Time taken: {elapsed_time} seconds\n\n")

if __name__ == '__main__':
    import sys
    
    for type_folder in ["utils", "core"]:
        log_path = os.path.join("./logs/", type_folder)
        target_folder = os.path.join("./benchmarks/", type_folder)
        run_circom_commands(target_folder, log_path)