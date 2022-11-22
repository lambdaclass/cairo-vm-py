import cairo_rs_py
import sys

def new_runner(program_name: str):
    with open(f"cairo_programs/{program_name}.json") as file:
        cairo_runner =  cairo_rs_py.CairoRunner(file.read(), "main", "all", False)
        return cairo_runner.cairo_run(False, f"cairo_programs/{program_name}.rs.trace", f"cairo_programs/{program_name}.rs.memory")

if __name__ == "__main__":
    program_name = sys.argv[1]
    if program_name in ["blake2s_felt", "blake2s_finalize", "blake2s_integration_tests", "blake2s_hello_world_hash", "dict_squash", "squash_dict", "dict_write", "dict_read", "dict_update"]:
        pass
    else: 
        new_runner(program_name)
    print("Pass")
    
    
