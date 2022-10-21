import cairo_rs_py

def assert_not_zero_hint():
    print(cairo_rs_py.cairo_run("cairo_programs/assert_not_zero.json", "main", False, False, None, None))

def memory_add_hint():
    print(cairo_rs_py.cairo_run("cairo_programs/memory_add.json", "main", False, False, None, None))

def print_vars_hint():
    print(cairo_rs_py.cairo_run("cairo_programs/hint_print_vars.json","main",False,False,None,None))
    
def scope_hints():
    print(cairo_rs_py.cairo_run("cairo_programs/vm_scope_hints.json","main",False,False,None,None))

def is_le_felt_hint():
    print(cairo_rs_py.cairo_run("cairo_programs/is_le_felt_hint.json", "main", False, False, None, None))

def assert_lt_felt_hint():
    print(cairo_rs_py.cairo_run("cairo_programs/assert_lt_felt_hint.json", "main", False, False, None, None))

def is_nn_hint():
    print(cairo_rs_py.cairo_run("cairo_programs/is_nn_hint.json", "main", False, False, None, None))

def is_nn_out_of_range_hint():
    print(cairo_rs_py.cairo_run("cairo_programs/is_nn_out_of_range_hint.json", "main", False, False, None, None))

def ec_mul_inner():
    print(cairo_rs_py.cairo_run("cairo_programs/ec_mul_inner.json","main",False,False,None,None))

def ec_negate():
    print(cairo_rs_py.cairo_run("cairo_programs/ec_negate.json","main",False,False,None,None))

def assert_nn_hint():
    print(cairo_rs_py.cairo_run("cairo_programs/assert_nn_hint.json", "main", False, False, None, None))

def assert_not_equal_hint():
    print(cairo_rs_py.cairo_run("cairo_programs/assert_not_equal_hint.json", "main", False, False, None, None))

def is_positive_hint():
    print(cairo_rs_py.cairo_run("cairo_programs/is_positive_hint.json", "main", False, False, None, None))

def pow_hint():
    print(cairo_rs_py.cairo_run("cairo_programs/pow_hint.json", "main", False, False, None, None))

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
    assert_not_equal_hint()
    # ec_negate()
    is_nn_out_of_range_hint()
    is_positive_hint()
    pow_hint()
    print("All test have passed")
