import sys
from utils import *

sys.setrecursionlimit(2000)

def append_log(new_log, log_path):
    if log_path is not None:
        with open(log_path, 'a') as file:
            file.write(new_log + "\n")
            

            
def main():
    log_path = None
    log_path_with_contributions = None
    if len(sys.argv) == 2:
        target_file_path = sys.argv[1]
    elif len(sys.argv) == 3:
        target_file_path = sys.argv[1]
        log_path = sys.argv[2]
    elif len(sys.argv) == 4:
        target_file_path = sys.argv[1]
        log_path = sys.argv[2]
        log_path_with_contributions = sys.argv[3]
    elif len(sys.argv) == 5:
        target_file_path = sys.argv[1]
        log_path = sys.argv[2]
        log_path_with_contributions = sys.argv[3]
        flag_str = sys.argv[4]
        assert len(flag_str) == 3
        flags_lst = []
        for one_bit in flag_str:
            assert one_bit in ['0', '1']
            if one_bit == '0':
                flags_lst.append(False)
            else:
                assert one_bit == '1'
                flags_lst.append(True)
        set_flags(flags_lst[0], flags_lst[1], flags_lst[2])
        
    else:
        assert len(sys.argv) == 6, "incorrect number of arguments"
        target_file_path = sys.argv[1]
        log_path = sys.argv[2]
        log_path_with_contributions = sys.argv[3]
        flag_str = sys.argv[4]
        assert len(flag_str) == 3
        flags_lst = []
        for one_bit in flag_str:
            assert one_bit in ['0', '1']
            if one_bit == '0':
                flags_lst.append(False)
            else:
                assert one_bit == '1'
                flags_lst.append(True)
                
        cur_max_depth = sys.argv[5]
        
        set_flags(flags_lst[0], flags_lst[1], flags_lst[2])
        set_bpg_depth(int(cur_max_depth))
        
        
        
    base_name = target_file_path.split('/')[-1].split('.')[0]
    
    try:
        message, time_taken, counterexample = main_solve(target_file_path)
    except Exception as e:
        message, time_taken, counterexample = e, 99999999, None
    
    
    log_message = ""
    log_message += f"** filename: {base_name}.circom \n"
    log_message += f"** result: {message} \n"
    log_message += f"** time: {time_taken} \n"
    log_message += f"** counterexample: {counterexample} \n"
    append_log(log_message, log_path)

    log_message += f"    ** contribution counts: \n"
    for cont in contribution_count:
        if cont == "bpg_time_records":
            log_message += f"    ****** {cont}: {contribution_count[cont]} \n"
        else:
            log_message += f"    ****** {cont}: {len(contribution_count[cont])} \n"
    append_log(log_message, log_path_with_contributions)

    return log_message

if __name__ == "__main__":
    main()