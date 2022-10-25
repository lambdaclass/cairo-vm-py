%builtins output range_check
from starkware.cairo.common.math import assert_le
from starkware.cairo.common.serialize import serialize_word


func signed_div_rem{range_check_ptr}(value, div, bound) -> (q : felt, r : felt):
    let r = [range_check_ptr]
    let biased_q = [range_check_ptr + 1]  # == q + bound.
    let range_check_ptr = range_check_ptr + 2
    %{
        # TEST
        from starkware.cairo.common.math_utils import as_int, assert_integer

        assert_integer(ids.div)
        assert 0 < ids.div <= PRIME // range_check_builtin.bound, \
            f'div={hex(ids.div)} is out of the valid range.'

        assert_integer(ids.bound)
        assert ids.bound <= range_check_builtin.bound // 2, \
            f'bound={hex(ids.bound)} is out of the valid range.'

        int_value = as_int(ids.value, PRIME)
        q, ids.r = divmod(int_value, ids.div)

        assert -ids.bound <= q < ids.bound, \
            f'{int_value} / {ids.div} = {q} is out of the range [{-ids.bound}, {ids.bound}).'

        ids.biased_q = q + ids.bound
    %}
    let q = biased_q - bound
    assert value = q * div + r
    assert_le(r, div - 1)
    assert_le(biased_q, 2 * bound - 1)
    return (q, r)
end

func main{output_ptr: felt*, range_check_ptr: felt}():
     let (q, r) = signed_div_rem(-10, 3, 29)

     assert q = 3618502788666131213697322783095070105623107215331596699973092056135872020477
     assert r = 2
     return ()
end
