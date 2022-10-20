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
    print(cairo_rs_py.cairo_run("cairo_programs/assert_le_felt_hint.json", "main", False, False, None, None))

def is_nn_hint():
    print(cairo_rs_py.cairo_run("cairo_programs/is_nn_hint.json", "main", False, False, None, None))
def ec_mul_inner():
    print(cairo_rs_py.cairo_run("cairo_programs/ec_mul_inner.json","main",False,False,None,None))

def ec_negate():
    print(cairo_rs_py.cairo_run("cairo_programs/ec_negate.json","main",False,False,None,None))

def dict_new():
    print(cairo_rs_py.cairo_run("cairo_programs/dict_new.json","main",False,False,None,None))

def dict_read():
    print(cairo_rs_py.cairo_run("cairo_programs/dict_read.json","main",False,False,None,None))

def dict_write():
    print(cairo_rs_py.cairo_run("cairo_programs/dict_write.json","main",False,False,None,None))

def dict_update():
    print(cairo_rs_py.cairo_run("cairo_programs/dict_update.json","main",False,False,None,None))
    

if __name__ == "__main__":
    assert_not_zero_hint()
    memory_add_hint()
    print_vars_hint()
    scope_hints()
    is_le_felt_hint()
    assert_lt_felt_hint()
    #is_nn_hint()
    ec_mul_inner()
    # ec_negate()
    dict_new()
    #dict_read()
    #dict_write()
    #dict_update()
    print("All test have passed")
