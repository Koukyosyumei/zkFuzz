import re
from misc import *



    
def get_contribution_results_from_logs(log_path):
    log_data = read_from_file(log_path).strip().split('\n')

    our_results = dict()

    filename = None
    result = None
    time_taken = None
    contributions = dict()

    for line in log_data:
        line = line.strip()
        if line.startswith('** filename: '):
            print((result,time_taken), filename)
            if filename is not None:

                assert filename not in our_results
                our_results[filename] = {
                    'result': result,
                    'time': time_taken,
                    'contributions': contributions
                }



            filename = line.split(" ")[-1]
            result = None
            time_taken = None
            contributions = dict()

        elif line.startswith('** time: '):
            time_taken = float(line.split(" ")[-1])

        elif line.startswith('** result: '):
            result = line.split(" ")[-1]
            if result == "CONSTRAINED!":
                result = "safe"
            elif result == "UNDER-CONSTRAINED!":
                result = "unsafe"
            else:
                assert result in ["TIMEOUT", "SURE"]
                result = "unknown"

                
        elif line.startswith('******'):
            parts = line.split(':')
            contribution_type = parts[0].strip('* ')
            if contribution_type == "bpg_time_records":
                count = eval(parts[1].strip())
            else:
                count = int(parts[1].strip())
            contributions[contribution_type] = count
            
            
            

    print((result,time_taken), filename)
    if filename is not None:
        assert filename not in our_results
        our_results[filename] = {
            'result': result,
            'time': time_taken,
            'contributions': contributions
        }


    return our_results




def get_solved_rate(our_results, all_circuit_stats):
    
    num_circuits_ALL = 0
    our_safe_ALL = 0
    our_unsafe_ALL = 0
    our_unknown_ALL = 0



    for type_bench in ['core', 'utils']:
        num_circuits_all = 0
        our_safe_all = 0
        our_unsafe_all = 0
        our_unknown_all = 0

        for cur_category in ["small", "medium", "large"]:
            num_circuits = 0
            our_safe = 0
            our_unsafe = 0
            our_unknown = 0
            for k in our_results:
                if all_circuit_stats[k]['category'] == cur_category and all_circuit_stats[k]['type_bench'] == type_bench:
                    if our_results[k]['result'] == 'safe':
                        our_safe += 1
                    elif our_results[k]['result'] == 'unsafe':
                        our_unsafe += 1
                    else:
                        assert our_results[k]['result'] == 'unknown'
                        our_unknown += 1

                    num_circuits += 1

            num_circuits_all += num_circuits
            our_safe_all +=our_safe
            our_unsafe_all +=our_unsafe
            our_unknown_all +=our_unknown


        num_circuits_ALL += num_circuits_all
        our_safe_ALL += our_safe_all
        our_unsafe_ALL += our_unsafe_all
        our_unknown_ALL += our_unknown_all


    return 1-our_unknown_ALL/num_circuits_ALL




def get_time(our_results, all_circuit_stats):

    our_total_time_type_ALL = 0
    num_circuits_ALL = 0

    for type_bench in ['core', 'utils']:
        num_circuits_all = 0
        our_total_time_type = 0

        for cur_category in ["small", "medium", "large"]:
            num_circuits = 0
            our_time = 0
            for k in our_results:
                if all_circuit_stats[k]['category'] == cur_category and all_circuit_stats[k]['type_bench'] == type_bench:
                    our_time += our_results[k]['time']
                    num_circuits += 1
            our_total_time_type += our_time
            num_circuits_all += num_circuits
            
        our_total_time_type_ALL += our_total_time_type
        num_circuits_ALL += num_circuits_all
    return our_total_time_type_ALL/num_circuits_ALL




def get_overall_average_time_accuracy_from_logs(log_path,all_circuit_stats):
    log_data = read_from_file(log_path).strip().split('\n')

    our_results = dict()

    filename = None
    result = None
    time_taken = None

    for line in log_data:
        line = line.strip()
        if line.startswith('** filename: '):
            if filename is not None:

                assert filename not in our_results
                cur_stats_d = dict()
                cur_stats_d['result'] = result
                cur_stats_d['time'] = time_taken

                our_results[filename] = cur_stats_d



            filename = line.split(" ")[-1]
            result = None
            time_taken = None

        if line.startswith('** time: '):
            time_taken = float(line.split(" ")[-1])

        if line.startswith('** result: '):
            result = line.split(" ")[-1]
            if result == "CONSTRAINED!":
                result = "safe"
            elif result == "UNDER-CONSTRAINED!":
                result = "unsafe"
            else:
                
                result = "unknown"


    if filename is not None:

        assert filename not in our_results
        cur_stats_d = dict()
        cur_stats_d['result'] = result
        cur_stats_d['time'] = time_taken

        our_results[filename] = cur_stats_d


    for k in our_results:
        if our_results[k]['time'] > 30:
            our_results[k]['time'] = 30
            our_results[k]['result'] = 'unknown'
            
    sr = get_solved_rate(our_results, all_circuit_stats)
    t = get_time(our_results, all_circuit_stats)   
    return sr, t


