%builtins range_check
from starkware.cairo.common.cairo_secp.bigint import (
    BigInt3,
    UnreducedBigInt3)
from nondet_bigint3 import nondet_bigint3
from starkware.cairo.common.cairo_secp.field import (
    unreduced_mul,
    verify_zero,
)
from starkware.cairo.common.cairo_secp.ec import EcPoint


func compute_slope{range_check_ptr}(point0 : EcPoint, point1 : EcPoint) -> (slope : BigInt3):
    %{
        # TEST
        from starkware.cairo.common.cairo_secp.secp_utils import SECP_P, pack
        from starkware.python.math_utils import line_slope

        # Compute the slope.
        x0 = pack(ids.point0.x, PRIME)
        y0 = pack(ids.point0.y, PRIME)
        x1 = pack(ids.point1.x, PRIME)
        y1 = pack(ids.point1.y, PRIME)
        value = slope = line_slope(point1=(x0, y0), point2=(x1, y1), p=SECP_P)
    %}
    let (slope) = nondet_bigint3()

    let x_diff = BigInt3(
        d0=point0.x.d0 - point1.x.d0, d1=point0.x.d1 - point1.x.d1, d2=point0.x.d2 - point1.x.d2
    )
    let (x_diff_slope : UnreducedBigInt3) = unreduced_mul(x_diff, slope)

    verify_zero(
        UnreducedBigInt3(
        d0=x_diff_slope.d0 - point0.y.d0 + point1.y.d0,
        d1=x_diff_slope.d1 - point0.y.d1 + point1.y.d1,
        d2=x_diff_slope.d2 - point0.y.d2 + point1.y.d2),
    )

    return (slope)
end

func main{range_check_ptr: felt}():

    #ec_negate
    let point_a = EcPoint(BigInt3(1, 5, 10), BigInt3(2, 4, 20))
    let point_b = EcPoint(BigInt3(1, 5, 10), BigInt3(77371252455336262886226989, 77371252455336267181195259, 19342813113834066795298795))
    let point_c = EcPoint(BigInt3(156, 6545, 100010), BigInt3(77371252455336262886225868, 1324, 19342813113834066795297906))

    #compute_slope
    let (slope_c) = compute_slope(point_a, point_c)
    assert slope_c = BigInt3(71370520431055565073514403,50503780757454603164423474, 8638166971146679236895064)

    let (slope_d) = compute_slope(point_c, point_b)
    assert slope_d = BigInt3(58119528729789858876194497,64998517253171473791555897, 16525667392681120436481221)

    return()
end
