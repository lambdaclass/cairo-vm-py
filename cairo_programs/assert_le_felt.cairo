%builtins range_check

from starkware.cairo.common.math import assert_nn_le, split_felt, assert_le

func assert_le_felt{range_check_ptr}(a, b):
    alloc_locals
    local small_inputs
    %{
        # TEST
        from starkware.cairo.common.math_utils import assert_integer
        assert_integer(ids.a)
        assert_integer(ids.b)
        a = ids.a % PRIME
        b = ids.b % PRIME
        assert a <= b, f'a = {a} is not less than or equal to b = {b}.'

        ids.small_inputs = int(
            a < range_check_builtin.bound and (b - a) < range_check_builtin.bound)
    %}
    if small_inputs != 0:
        assert_nn_le(a, b)
        ap += 33
        return ()
    end

    let (local a_high, local a_low) = split_felt(a)
    let (b_high, b_low) = split_felt(b)

    if a_high == b_high:
        assert_le(a_low, b_low)
        return ()
    end
    assert_le(a_high, b_high)
    return ()
end

func main{range_check_ptr : felt}():
    let x = 5
    let y = 6

    tempvar m = 7
    tempvar n = 7 * 7

    assert_le_felt(1, 2)
    assert_le_felt(2, 2)
    assert_le_felt(-2, -1)
    assert_le_felt(1, -1)
    assert_le_felt(0, 1)
    assert_le_felt(x, y)
    assert_le_felt(m, n)

    return ()
end
