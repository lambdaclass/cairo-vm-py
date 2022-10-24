%builtins range_check

from starkware.cairo.common.math import assert_le_felt, assert_lt_felt

const RC_BOUND = 2 ** 128

# Returns 1 if a >= 0 (or more precisely 0 <= a < RANGE_CHECK_BOUND).
# Returns 0 otherwise.
func is_nn{range_check_ptr}(a) -> (res : felt):
    %{ 
        # TEST
        memory[ap] = 0 if 0 <= (ids.a % PRIME) < range_check_builtin.bound else 1 
    %}
    jmp out_of_range if [ap] != 0; ap++
    [range_check_ptr] = a
    let range_check_ptr = range_check_ptr + 1
    return (res=1)

    out_of_range:
    %{ 
        # TEST
        memory[ap] = 0 if 0 <= ((-ids.a - 1) % PRIME) < range_check_builtin.bound else 1 
    %}
    jmp need_felt_comparison if [ap] != 0; ap++
    assert [range_check_ptr] = (-a) - 1
    let range_check_ptr = range_check_ptr + 1
    return (res=0)

    need_felt_comparison:
    assert_le_felt(RC_BOUND, a)
    return (res=0)
end

func main{range_check_ptr: felt}():
    #is_nn
    let (d) = is_nn(0)
    assert d = 1
    let (e) = is_nn(88)
    assert e = 1
    let (f) = is_nn(-88)
    assert f = 0
    return ()
end
