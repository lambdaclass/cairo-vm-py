import cairo_rs_py

def assert_not_zero_hint():
    print(cairo_rs_py.cairo_run("cairo_programs/assert_not_zero.json", "main", False, False, None, None))

def memory_add_hint():
    print(cairo_rs_py.cairo_run("cairo_programs/memory_add.json", "main", False, False, None, None))

def print_vars_hint():
    print(cairo_rs_py.cairo_run("cairo_programs/hint_print_vars.json","main",False,False,None,None))

def scope_hints():
    print(cairo_rs_py.cairo_run("cairo_programs/vm_scope_hints.json","main",False,False,None,None))

if __name__ == "__main__":
    assert_not_zero_hint()
    memory_add_hint()
    print_vars_hint()
    scope_hints()
    print("ok")
