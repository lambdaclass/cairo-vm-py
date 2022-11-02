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
    test_program("ec_negate")
    test_program("assert_nn_hint")
    test_program("pow")
    test_program("is_nn")
    test_program("is_positive")
    test_program("assert_not_zero")
    test_program("assert_le_felt")
    test_program("assert_lt_felt")
    test_program("assert_not_equal")
    test_program("reduce_and_nondet_bigint3")
    test_program("is_zero")
    test_program("div_mod_n")
    test_program("get_point_from_x")
    test_program("compute_slope")
    test_program("ec_doble")
    test_program("memcpy")
    test_program("memset")
    test_program("dict_new")
    test_program("dict_read")
    # test_program("dict_write") # ValueError: Custom Hint Error: AttributeError: 'PyTypeId' object has no attribute 'segment_index'
    test_program("dict_update")
    test_program("default_dict_new")
    # test_program("squash_dict") # ValueError: Custom Hint Error: ValueError: Failed to get ids value
    # test_program("dict_squash") # Custom Hint Error: AttributeError: 'PyTypeId' object has no attribute 'segment_index'
    test_program("ids_size")
    test_program("split_felt") 
    test_program("split_int")
    test_program("split_64")
    test_program("uint256_add")
    test_program("uint256_sqrt")
    test_program("uint256_unsigned_div_rem")
    test_program("uint256_signed_nn")
    test_program("bigint_to_uint256")
    test_program("usort")
    test_program("sqrt")
    test_program("unsigned_div_rem")
    test_program("signed_div_rem")
    test_program("find_element")
    test_program("search_sorted_lower")
    # test_program("set_add") # Custom Hint Error: AttributeError: 'builtins.PyMemory' object has no attribute 'get_range'
    test_program("assert_250_bit")
    test_program("blake2s_hello_world_hash") # ValueError: Custom Hint Error: AttributeError: 'builtins.MemorySegmentManager' object has no attribute 'memory'
    test_program("blake2s_finalize") # ValueError: Custom Hint Error: AttributeError: 'builtins.MemorySegmentManager' object has no attribute 'memory'
    test_program("blake2s_felt") # ValueError: Custom Hint Error: AttributeError: 'builtins.MemorySegmentManager' object has no attribute 'memory'
    print("\nAll test have passed")
