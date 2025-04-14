import os
import yaml


############################## Mod Utils ##############################

p = 21888242871839275222246405745257275088548364400416034343698204186575808495617


def get_constant_gf_p(c):
    return (c+p) % p


def check_constant(c):
    assert c >= 0
    assert c < p


def check_var_pair(var_pair):
    assert isinstance(var_pair, tuple)
    if len(var_pair) == 2:
        assert var_pair[0] <= var_pair[1]
    else:
        assert len(var_pair) in [0, 1]
    return True


def add_constants(const1, const2):
    check_constant(const1)
    check_constant(const2)
    
    return (const1 + const2 + p) % p



def mul_constants(const1, const2):
    check_constant(const1)
    check_constant(const2)
    
    return (const1 * const2 + p) % p


def invert_constant(const):
    # Fermat's little theorem
    check_constant(const)
    assert const != 0, "cannot invert 0"
    
    return pow(const, p - 2, p)


def negate_constant(const):
    check_constant(const)
    const = const % p
    
    return (-const + p) % p


def pow_constant(base, exponent):
    return pow(base , exponent , p)



def has_sqrt_mod_p(N):
    return legendre_symbol(N) in [0, 1]

def legendre_symbol(a):
    if a % p == 0:
        return 0
    ls = pow(a, (p - 1) // 2, p)
    return -1 if ls == p - 1 else ls



def find_non_residue():
    for n in range(2, p):
        if legendre_symbol(n) == -1:
            return n
    raise ValueError("Cannot find a quadratic non-residue")
    
    
def sqrt_mod_p(a):
    # Tonelli-Shanks algorithm
    if a == 0:
        return 0, 0
    
    assert has_sqrt_mod_p(a), "Not a square (mod p)"
    Q = p - 1
    S = 0
    while Q % 2 == 0:
        Q //= 2
        S += 1
        
    # now, Q is odd, p-1 is Q * (2**S)
    Z = find_non_residue() 
    M = S
    c = pow(Z, Q, p)
    t = pow(a, Q, p)
    R = pow(a, (Q + 1) // 2, p)
    
    while t != 1:
        i = 1
        t2i = pow(t, 2, p)
        while t2i != 1:
            t2i = (t2i * t2i) % p
            i += 1
        b = pow(c, 2**(M - i - 1), p)
        M = i
        c = (b * b) % p
        t = (t * c) % p
        R = (R * b) % p
        
        
    root1 = R
    root2 = p - R
    check_constant(root1)
    check_constant(root2)
    return root1, root2  



 

    
def solve_quadratic_equation_in_Fp(a, b, c):
    # Solves ax^2 + bx + c = 0 in GF(p)
    # Returns: found_solution, solution
    
    check_constant(a)
    check_constant(b)
    check_constant(c)
    
    # a != 0: only consider quadratics
    # b != 0: pure squares are solved in a separate function
    assert a != 0 and b != 0

    b = mul_constants(b, invert_constant(a))
    c = mul_constants(c, invert_constant(a))

    b_half = mul_constants(b, invert_constant(2))
    b_half_squared = mul_constants(b_half, b_half)
    D = add_constants(b_half_squared, negate_constant(c))
    
    
    if has_sqrt_mod_p(D):
        sqrt_D1, sqrt_D2 = sqrt_mod_p(D)
        
        negative_b_half = negate_constant(b_half)
        
        x1 = add_constants(negative_b_half, sqrt_D1)
        x2 = add_constants(negative_b_half, sqrt_D2)
        
        return True, (x1, x2)
    else:
        return False, None

############################## I/O Utils ##############################


def write_to_file(contents, path):
    with open(path, 'w') as f:
        f.write(contents)

def read_from_file(path):
    with open(path) as f:
        return ''.join(f.readlines())

    
def save_data_structure(data_structure, save_to_file_name):
    assert save_to_file_name.endswith('.yaml')
    with open(save_to_file_name, 'w') as outfile:
        yaml.safe_dump(data_structure, outfile, default_flow_style=False)

        
def load_data_structure(read_from_file_name):
    assert read_from_file_name.endswith('.yaml')
    with open(read_from_file_name, 'r') as file:
        data = yaml.safe_load(file)
    return data    
    
    

    
    
############################## Others ##############################
 

def dictify_all_paths(all_paths):
    # input is a list of (var, property)
    # returns a map that maps vars to a set of property tuples of that var
    result = dict()
    for item in all_paths:
        key = item[0]
        if key not in result:
            result[key] = set()
        result[key].add(item)
    return result
    
    