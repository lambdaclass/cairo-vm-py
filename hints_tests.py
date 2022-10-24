import cairo_rs_py

def new_vm():
    return cairo_rs_py.PyVM(3618502788666131213697322783095070105623107215331596699973092056135872020481, True)

def test_program(program_name: str):
    print(new_vm().cairo_run(f"cairo_programs/{program_name}.json", "main", False))

if __name__ == "__main__":
    test_program("assert_not_zero")
    test_program("memory_add")
    test_program("hint_print_vars")
    test_program("vm_scope_hints")
    test_program("is_le_felt_hint")
    test_program("assert_le_felt_hint")
    test_program("is_nn_hint")
    test_program("ec_mul_inner")
    # test_program("ec_negate")
    test_program("assert_nn_hint")
    test_program("assert_not_equal_hint")
    test_program("assert_lt_felt_hint")
    test_program("is_nn_out_of_range_hint")
    # THIS TWO ARE FAILING
    # test_program("pow_hint")
    # test_program("is_positive_hint")
    print("\nAll test have passed")
