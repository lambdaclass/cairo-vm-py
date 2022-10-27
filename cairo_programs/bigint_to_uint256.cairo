%builtins range_check
from starkware.cairo.common.uint256 import Uint256
from starkware.cairo.common.cairo_secp.constants import BASE
from starkware.cairo.common.cairo_secp.bigint import BigInt3
from starkware.cairo.common.math_cmp import RC_BOUND
from starkware.cairo.common.math import assert_nn_le

func bigint_to_uint256{range_check_ptr}(x : BigInt3) -> (res : Uint256):
    let low = [range_check_ptr]
    let high = [range_check_ptr + 1]
    let range_check_ptr = range_check_ptr + 2
    %{ 
        #TEST
        ids.low = (ids.x.d0 + ids.x.d1 * ids.BASE) & ((1 << 128) - 1) 
    %}
    # Because PRIME is at least 174 bits, the numerator doesn't overflow.
    tempvar a = ((x.d0 + x.d1 * BASE) - low) / RC_BOUND
    const D2_SHIFT = BASE * BASE / RC_BOUND
    const A_BOUND = 4 * D2_SHIFT
    # We'll check that the division in `a` doesn't cause an overflow. This means that the 128 LSB
    # of (x.d0 + x.d1 * BASE) and low are identical, which ensures that low is correct.
    assert_nn_le(a, A_BOUND - 1)
    # high * RC_BOUND = a * RC_BOUND + x.d2 * BASE ** 2 =
    #   = x.d0 + x.d1 * BASE + x.d2 * BASE ** 2 - low = num - low.
    with_attr error_message("x out of range"):
        assert high = a + x.d2 * D2_SHIFT
    end

    return (res=Uint256(low=low, high=high))
end

func main{range_check_ptr}():
    let n = BigInt3(1,0,0)
    let (r) = bigint_to_uint256(n)
    assert r = Uint256(1,0)
    return()
end
