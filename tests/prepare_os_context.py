import cairo_rs_py

def init_runner(program_name):
    return cairo_rs_py.CairoRunner(f"cairo_programs/{program_name}.json", "main", "small", False)


def prepare_os_context(runner):
    syscall_segment = runner.add_segment()
    os_context = [syscall_segment]
    os_context.extend(runner.get_builtins_initial_stack())
    return os_context


if __name__ == "__main__":
    runner = init_runner("get_builtins_initial_stack")
    context = prepare_os_context(runner)
    assert str(context) == str([(0,0)])
    print("prepare_os_context test passed")