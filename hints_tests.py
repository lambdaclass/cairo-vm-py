import cairo_rs_py

def assert_not_zero_hint():
    print(cairo_rs_py.cairo_run("cairo_programs/assert_not_zero.json", "main", False, False, None, None))
def memory_add_hint():
    print(cairo_rs_py.cairo_run("cairo_programs/memory_add.json", "main", False, False, None, None))

def print_vars_hint():
    print(cairo_rs_py.cairo_run("cairo_programs/hint_print_vars.json","main",False,False,None,None))

def ec_mul_inner():
    print(cairo_rs_py.cairo_run("cairo_programs/ec_mul_inner.json","main",False,False,None,None))

def ec_negate():
    print(cairo_rs_py.cairo_run("cairo_programs/ec_negate.json","main",False,False,None,None))

if __name__ == "__main__":
    assert_not_zero_hint()
    memory_add_hint()
    print_vars_hint()
    ec_mul_inner()
    # ec_negate()
    print("All test have passed")