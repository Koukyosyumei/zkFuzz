import struct
import sys
import subprocess
import os
import time


# Acknowledgements and Credits

# The utilities for parsing and reading R1CS files in this source code were adapted from 
# the implementation found in the Picus project, originally developed by Yanju Chen and 
# others. The original code is written in Racket and is available at:
# https://github.com/chyanju/Picus
# (Picus is licensed under the MIT License.)

# Reference information of the source
# @article{yanju2023automated,
# author = {Pailoor, Shankara and Chen, Yanju and Wang, Franklyn and Rodr\'{\i}guez, Clara and Van Geffen, Jacob and Morton, Jason and Chu, Michael and Gu, Brian and Feng, Yu and Dillig, I\c{s}\i{}l},
# title = {Automated Detection of Under-Constrained Circuits in Zero-Knowledge Proofs},
# year = {2023},
# issue_date = {June 2023},
# publisher = {Association for Computing Machinery},
# address = {New York, NY, USA},
# volume = {7},
# number = {PLDI},
# url = {https://doi.org/10.1145/3591282},
# doi = {10.1145/3591282},
# journal = {Proc. ACM Program. Lang.},
# month = {jun},
# articleno = {168},
# numpages = {23},
# keywords = {SNARKs, program verification, zero-knowledge proofs}
# } 

# This code also references specifications for the R1CS binary file format, as described in:
# https://github.com/iden3/r1csfile/blob/master/doc/r1cs_bin_format.md




p = 21888242871839275222246405745257275088548364400416034343698204186575808495617


class ConstraintBlock:
    def __init__(self, nnz, wids, factors):
        self.nnz = nnz  
        self.wids = wids 
        self.factors = factors 

    def __str__(self):
        return f"ConstraintBlock(nnz={self.nnz}, wids={self.wids}, factors={self.factors})"

class Constraint:
    def __init__(self, block_a, block_b, block_c):
        self.block_a = block_a  
        self.block_b = block_b  
        self.block_c = block_c 

    def __str__(self):
        return f"Constraint(block_a={self.block_a}, block_b={self.block_b}, block_c={self.block_c})"

    
class W2LSection:
    def __init__(self, numbers):
        self.numbers = numbers 
    
    
class HeaderVariables:
    def __init__(self, field_size, prime_number, nwires, npubout, npubin, nprvin, nlabels, mconstraints):
        self.field_size = field_size
        self.prime_number = prime_number
        self.nwires = nwires
        self.npubout = npubout
        self.npubin = npubin
        self.nprvin = nprvin
        self.nlabels = nlabels
        self.mconstraints = mconstraints
        
        
class R1CS:
    def __init__(self, magic_number, version, num_sections, header_vars, constraint_section, wireid2label_section, input_list, output_list):
        self.magic_number = magic_number
        self.version = version
        self.num_sections = num_sections
        self.header_vars = header_vars
        self.constraint_section = constraint_section
        self.wireid2label_section = wireid2label_section
        self.input_list = input_list
        self.output_list = output_list
        
        
    def print_constraint_block_ABC(self, constraint_id):
        sample_constraint = self.parsed_constraints_ABC[constraint_id]
        print("Ax:", sample_constraint['A'])
        print("Bx:", sample_constraint['B'])
        print("Cx:", sample_constraint['C'])
        
        
    def print_constraint_block_whole_line(self):
        for constraint_id, constraint in enumerate(self.parsed_constraints_whole_line):
            print(f"Constraint {constraint_id}: {constraint}")
        
    
def filter_sections(arg_raw, accepted_types = [1, 2, 3]):
    if len(arg_raw) == 0:
        return bytes()

    section0_type = struct.unpack('<I', arg_raw[0:4])[0]
    section0_size = struct.unpack('<Q', arg_raw[4:12])[0]  
    bs0 = 12 + section0_size 

    if section0_type in accepted_types:
        return arg_raw[0:bs0] + filter_sections(arg_raw[bs0:])
    else:
        return filter_sections(arg_raw[bs0:])


    
def count_sections(arg_raw):
    if len(arg_raw) == 0:
        return 0

    section0_type = struct.unpack('<I', arg_raw[0:4])[0]
    section0_size = struct.unpack('<Q', arg_raw[4:12])[0]  
    bs0 = 12 + section0_size 

    return 1 + count_sections(arg_raw[bs0:])


def find_section(arg_raw, arg_type):
    if len(arg_raw) == 0:
        raise Exception(f"# [exception][find-section-pos] cannot find position of section given type: {arg_type}.")

    section0_type = struct.unpack('<I', arg_raw[0:4])[0]
    section0_size = struct.unpack('<Q', arg_raw[4:12])[0]  

    if arg_type == section0_type:
        return 0, section0_size  # found
    else:
        offset = 12 + section0_size
        pos0, size0 = find_section(arg_raw[offset:], arg_type)
        return offset + pos0, size0

def extract_header_section(arg_raw):
    field_size = struct.unpack('<I', arg_raw[0:4])[0]  
    
    if field_size % 8 != 0:
        raise Exception()

    prime_number = arg_raw[4:4+field_size]  
    nwires = struct.unpack('<I', arg_raw[4+field_size:8+field_size])[0]
    npubout = struct.unpack('<I', arg_raw[8+field_size:12+field_size])[0]
    npubin = struct.unpack('<I', arg_raw[12+field_size:16+field_size])[0]
    nprvin = struct.unpack('<I', arg_raw[16+field_size:20+field_size])[0]
    nlabels = struct.unpack('<Q', arg_raw[20+field_size:28+field_size])[0] 
    mconstraints = struct.unpack('<I', arg_raw[28+field_size:32+field_size])[0]
    return HeaderVariables(field_size, prime_number, nwires, npubout, npubin, nprvin, nlabels, mconstraints)


def int_unpack_from_bytes(binary_data, byte_order='little'):
    if not isinstance(binary_data, bytes):
        raise ValueError("Input must be bytes.")
    
    return int.from_bytes(binary_data, byte_order)


def extract_single_constraint(arg_raw, arg_fs):
    def extract_constraint_block(arg_block, arg_n):
        tmp_wids = []
        tmp_factors = []
        for i in range(arg_n):
            s0 = i * (4 + arg_fs)
            tmp_wids.append(struct.unpack('<I', arg_block[s0:s0 + 4])[0])
            s0 = 4 + s0 
            assert arg_fs % 4 == 0
            tmp_factors.append(int_unpack_from_bytes(arg_block[s0:s0 + arg_fs]))
        return tmp_wids, tmp_factors


    nnz_a = struct.unpack('<I', arg_raw[0:4])[0] 
    block_a_start = 4
    block_a_end = block_a_start + nnz_a * (4 + arg_fs)
    block_a = arg_raw[block_a_start:block_a_end]  
    wids_a, factors_a = extract_constraint_block(block_a, nnz_a)

    block_b_start = block_a_end + 4
    nnz_b = struct.unpack('<I', arg_raw[block_a_end:block_b_start])[0]
    block_b_end = block_b_start + nnz_b * (4 + arg_fs)
    block_b = arg_raw[block_b_start:block_b_end]
    wids_b, factors_b = extract_constraint_block(block_b, nnz_b)

    block_c_start = block_b_end + 4
    nnz_c = struct.unpack('<I', arg_raw[block_b_end:block_c_start])[0]
    block_c_end = block_c_start + nnz_c * (4 + arg_fs)
    block_c = arg_raw[block_c_start:block_c_end]
    wids_c, factors_c = extract_constraint_block(block_c, nnz_c)

    ret0 = Constraint(
        ConstraintBlock(nnz_a, wids_a, factors_a),
        ConstraintBlock(nnz_b, wids_b, factors_b),
        ConstraintBlock(nnz_c, wids_c, factors_c)
    )

    return block_c_end, ret0



def extract_constraint_section(arg_raw, arg_fs, arg_m, timeout_time, starting_time):
    clist = []
    raw0 = arg_raw
    i = 0
    while len(raw0) > 0:
        i += 1
        if i % 50 == 0:
            if timeout_time is not None:
                assert starting_time is not None
                if time.time() - starting_time > timeout_time:
                    raise TimeoutError("The operation took too long")
                
        block_end, cs = extract_single_constraint(raw0, arg_fs)
        clist.append(cs)
        raw0 = raw0[block_end:]

    if len(clist) != arg_m:
        raise Exception(f"# [exception][extract-constraint-section] number of constraints is not equal to mconstraints, got: {len(clist)} and {arg_m}.")

    return clist




def extract_w2l_section(arg_raw):
    if len(arg_raw) % 8 != 0:
        raise Exception(f"# [exception][extract-w2l-section] bytes length should be a multiple of 8, got: {len(arg_raw)}.")

    n = len(arg_raw) // 8
    map0 = []
    for i in range(n):
        s0 = i * 8
        map0.append(struct.unpack('<Q', arg_raw[s0:s0 + 8])[0])

    return W2LSection(map0)

def read_r1cs(filename, timeout_time, starting_time):


    with open(filename, 'rb') as file:
        magic_number = file.read(4)
        if magic_number != bytes([0x72, 0x31, 0x63, 0x73]):
            print(f"# [exception][read-r1cs] magic number is incorrect, got: {magic_number}")

        version = file.read(4) 
        version = struct.unpack('<I', version)[0]
        if version != 1:
            print(f"# [exception][read-r1cs] version is not supported, got: {version}")

        num_sections = file.read(4)
        num_sections = struct.unpack('<I', num_sections)[0]

        raw_sections = file.read()

        fraw_sections = filter_sections(raw_sections) 
        if count_sections(fraw_sections) != 3:
            raise Exception(f"# [exception][read-r1cs] r1cs needs to contain 3 sections, got: {count_sections(fraw_sections)}")

        header_starting_index, header_size = find_section(fraw_sections, 1) 
        header_vars = extract_header_section(fraw_sections[header_starting_index + 12: header_starting_index + header_size + 12])
        field_size = header_vars.field_size


        constraint_starting_index, constraint_size = find_section(fraw_sections, 2)
        constraint_section = extract_constraint_section(
            fraw_sections[constraint_starting_index + 12: constraint_starting_index + constraint_size + 12], field_size, header_vars.mconstraints, timeout_time, starting_time
        )

        wireid2label_starting_index, wireid2label_size = find_section(fraw_sections, 3)
        wireid2label_section = extract_w2l_section(fraw_sections[wireid2label_starting_index + 12: wireid2label_starting_index + wireid2label_size + 12])


        istart = 2 + header_vars.npubout  
        iend = 1 + header_vars.npubout + header_vars.npubin + header_vars.nprvin  
        input_list_ecne = [1] + list(range(istart, iend + 1))
        input_list = [i - 1 for i in input_list_ecne]  

        ostart = 2  
        oend = 1 + header_vars.npubout 
        output_list_ecne = list(range(ostart, oend + 1))
        output_list = [i - 1 for i in output_list_ecne] 

        r1cs = R1CS(magic_number, version, num_sections, header_vars, constraint_section, wireid2label_section, input_list, output_list)
    return r1cs



def r1cs_to_string(r1cs, arg_id):
    constraint_section = r1cs.constraint_section  

    example_constraint = constraint_section[arg_id]  
    example_block_a = example_constraint.block_a
    example_block_b = example_constraint.block_b
    example_block_c = example_constraint.block_c

    nnz_a = example_block_a.nnz
    wids_a = example_block_a.wids
    factors_a = example_block_a.factors
    str_a = ' + '.join([
        f"({f} * x{w})"
        for w, f in zip(wids_a, factors_a)
    ])

    nnz_b = example_block_b.nnz
    wids_b = example_block_b.wids
    factors_b = example_block_b.factors
    str_b = ' + '.join([
        f"({f} * x{w})"
        for w, f in zip(wids_b, factors_b)
    ])

    nnz_c = example_block_c.nnz
    wids_c = example_block_c.wids
    factors_c = example_block_c.factors
    str_c = ' + '.join([
        f"({f} * x{w})"
        for w, f in zip(wids_c, factors_c)
    ])

    return f"( {(str_a if str_a else '0')} ) * ( {(str_b if str_b else '0')} ) = {(str_c if str_c else '0')}"





            



