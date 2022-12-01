import sys
import cairo_rs_py

def new_runner(program_name: str):
    with open(f"cairo_programs/bad_programs/{program_name}.json") as file:
        return cairo_rs_py.CairoRunner(file.read(), "main", "all", False)


def test_program_error(program_name: str, error_msg: str):
    try:
         new_runner(program_name).cairo_run(False)
         print(f"Failure {program_name} ran without errors")
         sys.exit(1)  
    except Exception as err:
        assert str(err).__contains__(error_msg), True
        print(f"{program_name} OK")

if __name__ == "__main__":

    test_program_error("error_msg_attr", "SafeUint256: addition overflow")

    print("\nAll test have passed")
