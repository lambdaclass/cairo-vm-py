%builtins range_check
from starkware.cairo.common.cairo_secp.ec import (
        EcPoint
)
from starkware.cairo.common.cairo_secp.bigint import BigInt3, UnreducedBigInt3
from starkware.cairo.common.cairo_secp.field import (
    verify_zero
)
from nondet_bigint3 import nondet_bigint3

func ec_negate{range_check_ptr}(point : EcPoint) -> (point : EcPoint):
    %{
        #TEST
        from starkware.cairo.common.cairo_secp.secp_utils import SECP_P, pack

        y = pack(ids.point.y, PRIME) % SECP_P
        # The modulo operation in python always returns a nonnegative number.
        value = (-y) % SECP_P
    %}

    let (minus_y) = nondet_bigint3()
    verify_zero(
        UnreducedBigInt3(
        d0=minus_y.d0 + point.y.d0,
        d1=minus_y.d1 + point.y.d1,
        d2=minus_y.d2 + point.y.d2),
    )

    return (point=EcPoint(x=point.x, y=minus_y))
end

func main{range_check_ptr: felt}():

    let x = BigInt3(1, 5, 10)
    let y = BigInt3(2, 4, 20)

    #ec_negate
    let point_a = EcPoint(x, y)
    let (point_b) = ec_negate(point_a)

    assert point_b = EcPoint(BigInt3(1, 5, 10), BigInt3(77371252455336262886226989, 77371252455336267181195259, 19342813113834066795298795))

    return()
end
