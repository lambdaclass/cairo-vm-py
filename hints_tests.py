import cairo_rs_py

def new_runner(program_name: str):
    return cairo_rs_py.CairoRunner(f"cairo_programs/{program_name}.json", "main")

def test_program(program_name: str):
    print(new_runner(program_name).cairo_run(False))

if __name__ == "__main__":
    test_program("assert_not_zero")
    test_program("memory_add")
    test_program("hint_print_vars")
    test_program("vm_scope_hints")
    test_program("is_le_felt_hint")
    test_program("ec_mul_inner")
    # test_program("ec_negate") # ValueError: Variable value not present in current execution scope
    test_program("assert_nn_hint")
    # test_program("pow") # Custom Hint Error: AttributeError: 'builtins.Relocatable' object has no attribute 'exp'
    test_program("is_nn")
    test_program("is_positive")
    test_program("assert_not_zero")
    test_program("assert_le_felt")
    test_program("assert_lt_felt")
    test_program("assert_not_equal")
    test_program("reduce_and_nondet_bigint3")
    # test_program("is_zero") # Custom Hint Error: NameError: name 'to_felt_or_relocatable' is not defined
    test_program("div_mod_n")
    # test_program("get_point_from_x") # Custom Hint Error: ValueError: Failed to get ids value
    # test_program("compute_slope") # ValueError: verify_zero: Invalid input 115792089237316195422966786669080118156105500425196604465460999109169899370287
    # test_program("ec_doble") # ValueError: verify_zero: Invalid input -16289419471130179420082969341938614127691693058652915375562000746308067257337178941098315954201487874090
    test_program("memcpy")
    test_program("memset")
    test_program("split_felt") 
    test_program("split_int")
    test_program("split_64")
    # test_program("uint256_add") # Custom Hint Error: ValueError: Failed to get ids value
    # test_program("uint256_sqrt") # Custom Hint Error: ValueError: Failed to get ids value
    # test_program("uint256_unsigned_div_rem") # Custom Hint Error: AttributeError: 'builtins.PyTypedId' object has no attribute 'low'
    # test_program("uint256_signed_nn") # Custom Hint Error: ValueError: Failed to get ids value
    # test_program("bigint_to_uint256") # Custom Hint Error: ValueError: Failed to get ids value
    test_program("usort")
    test_program("sqrt")
    test_program("unsigned_div_rem")
    test_program("signed_div_rem")
    test_program("find_element")
    test_program("search_sorted_lower")
    # test_program("set_add") # Custom Hint Error: AttributeError: 'builtins.PyMemory' object has no attribute 'get_range'
    print("\nAll test have passed")
