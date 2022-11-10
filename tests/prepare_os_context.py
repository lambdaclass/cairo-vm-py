import cairo_rs_py

def init_runner(program_name):
    with open(f"cairo_programs/{program_name}.json") as file:
        return cairo_rs_py.CairoRunner(file.read(), "main", "small", False)


def prepare_os_context_without_builtins(runner):
    syscall_segment = runner.add_segment()
    os_context = [syscall_segment]
    os_context.extend(runner.get_builtins_initial_stack())
    return os_context

def prepare_os_context_with_builtins(runner):
    syscall_segment = runner.add_segment()
    os_context = [syscall_segment]
    runner.initialize_function_runner()
    os_context.extend(runner.get_builtins_initial_stack())
    return os_context


if __name__ == "__main__":
    runner = init_runner("get_builtins_initial_stack")
    print("initialize no builtin")
    expected_output = [(0,0)]
    context = prepare_os_context_without_builtins(runner)
    assert str(context) == str(expected_output)

    print("initialize builtins")
    runner = init_runner("get_builtins_initial_stack")
    context = prepare_os_context_with_builtins(runner)
    expected_output =  [(0, 0), ('output', [(3, 0)]), ('pedersen', [(4, 0)]), ('range_check', [(5, 0)]), ('bitwise', [(6, 0)]), ('ec_op', [(7, 0)])]
    assert str(context) == str(expected_output)
    print("prepare_os_context tests passed")
