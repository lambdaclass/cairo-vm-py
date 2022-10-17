import cairo_rs_py

def assert_not_zero_hint():
    cairo_rs_py.cairo_run("cairo_programs/assert_not_zero.json", "main", False, False, None, None)

if __name__ == "__main__":
    assert_not_zero_hint()
    print("ok")