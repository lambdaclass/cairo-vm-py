import cairo_rs_py

def new_vm():
    return cairo_rs_py.PyVM(3618502788666131213697322783095070105623107215331596699973092056135872020481, True)

def assert_not_zero_hint():
    print(new_vm().cairo_run("cairo_programs/assert_not_zero.json", "main", False))

def memory_add_hint():
    print(new_vm().cairo_run("cairo_programs/memory_add.json", "main", False))

def print_vars_hint():
    print(new_vm().cairo_run("cairo_programs/hint_print_vars.json","main",False))
    
def scope_hints():
    print(new_vm().cairo_run("cairo_programs/vm_scope_hints.json","main",False))

def is_le_felt_hint():
    print(new_vm().cairo_run("cairo_programs/is_le_felt_hint.json", "main", False))

def assert_lt_felt_hint():
    print(new_vm().cairo_run("cairo_programs/assert_le_felt_hint.json", "main", False))

def is_nn_hint():
    print(new_vm().cairo_run("cairo_programs/is_nn_hint.json", "main", False))
    
def ec_mul_inner():
    print(new_vm().cairo_run("cairo_programs/ec_mul_inner.json","main",False))

def ec_negate():
    print(new_vm().cairo_run("cairo_programs/ec_negate.json","main",False))

def assert_nn_hint():
    print(new_vm().cairo_run("cairo_programs/assert_nn_hint.json", "main", False))

if __name__ == "__main__":
    assert_not_zero_hint()
    memory_add_hint()
    print_vars_hint()
    scope_hints()
    is_le_felt_hint()
    assert_lt_felt_hint()
    is_nn_hint()
    ec_mul_inner()
    assert_nn_hint()
    # ec_negate()
    print("All test have passed")
