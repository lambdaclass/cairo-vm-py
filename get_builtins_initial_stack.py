import cairo_rs_py

def new_runner(program_name: str):
    with open(f"cairo_programs/{program_name}.json") as file:
        return cairo_rs_py.CairoRunner(file.read(), "main", "all", False)

def test_program(program_name: str):
    runner = new_runner(program_name)
    
    builtins_initial_stack = runner.get_program_builtins_initial_stack()
    assert builtins_initial_stack == [], 'Initial stack should be empty.'
    
    runner.cairo_run(False)
    
    builtins_final_stack = runner.get_program_builtins_initial_stack()
    
    expected_output = [(2, 0)]
    assert str(builtins_final_stack) == str(expected_output)

if __name__ == "__main__":
    test_program("get_builtins_initial_stack")
    print("\nget_builtins_initial_stack test passed.")
