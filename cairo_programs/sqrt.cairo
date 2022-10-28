# Returns the floor value of the square root of the given value.
# Assumptions: 0 <= value < 2**250.
%builtins range_check

from starkware.cairo.common.math import assert_nn_le, assert_in_range

func sqrt{range_check_ptr}(value) -> (res : felt):
    alloc_locals
    local root : felt

    %{
        # TEST
        from starkware.python.math_utils import isqrt
        value = ids.value % PRIME
        assert value < 2 ** 250, f"value={value} is outside of the range [0, 2**250)."
        assert 2 ** 250 < PRIME
        ids.root = isqrt(value)
    %}

    assert_nn_le(root, 2 ** 125 - 1)
    tempvar root_plus_one = root + 1
    assert_in_range(value, root * root, root_plus_one * root_plus_one)

    return (res=root)
end

func main{range_check_ptr: felt}():
    let (result_a) = sqrt(0)
    assert result_a = 0

    let (result_b) = sqrt(2402)
    assert result_b = 49

    let (result_c) = sqrt(361850278866613121369732278309507010562)
    assert result_c = 19022362599493605525

    return()
end
