%builtins range_check

from starkware.cairo.common.cairo_secp.bigint import BigInt3, UnreducedBigInt3
from starkware.cairo.common.cairo_secp.field import unreduced_mul
from starkware.cairo.common.cairo_secp.field import verify_zero
from nondet_bigint3 import nondet_bigint3


func is_zero{range_check_ptr}(x : BigInt3) -> (res : felt):
    %{
        # TEST
        from starkware.cairo.common.cairo_secp.secp_utils import SECP_P, pack

        x = pack(ids.x, PRIME) % SECP_P
    %}
    if nondet %{ # TEST
                x == 0  %} != 0:
        verify_zero(UnreducedBigInt3(d0=x.d0, d1=x.d1, d2=x.d2))
        return (res=1)
    end

    %{
        # TEST
        from starkware.cairo.common.cairo_secp.secp_utils import SECP_P
        from starkware.python.math_utils import div_mod

        value = x_inv = div_mod(1, x, SECP_P)
    %}
    let (x_inv) = nondet_bigint3()
    let (x_x_inv) = unreduced_mul(x, x_inv)

    # Check that x * x_inv = 1 to verify that x != 0.
    verify_zero(UnreducedBigInt3(
        d0=x_x_inv.d0 - 1,
        d1=x_x_inv.d1,
        d2=x_x_inv.d2))
    return (res=0)
end

func main{range_check_ptr}():
    # is_zero
    let (u) = is_zero(BigInt3(0,0,0))
    assert u = 1
    let (v) = is_zero(BigInt3(232113757366008801543585,232113757366008801543585,232113757366008801543585))
    assert v = 0

    return ()
end
