%builtins range_check
from nondet_bigint3 import nondet_bigint3
from starkware.cairo.common.cairo_secp.bigint import BigInt3, UnreducedBigInt3
from starkware.cairo.common.cairo_secp.field import is_zero, unreduced_mul, unreduced_sqr, verify_zero
from starkware.cairo.common.cairo_secp.ec import EcPoint, ec_negate


func compute_doubling_slope{range_check_ptr}(point : EcPoint) -> (slope : BigInt3):
    # Note that y cannot be zero: assume that it is, then point = -point, so 2 * point = 0, which
    # contradicts the fact that the size of the curve is odd.
    %{
        # TEST
        from starkware.cairo.common.cairo_secp.secp_utils import SECP_P, pack
        from starkware.python.math_utils import ec_double_slope

        # Compute the slope.
        x = pack(ids.point.x, PRIME)
        y = pack(ids.point.y, PRIME)
        value = slope = ec_double_slope(point=(x, y), alpha=0, p=SECP_P)
    %}
    let (slope : BigInt3) = nondet_bigint3()

    let (x_sqr : UnreducedBigInt3) = unreduced_sqr(point.x)
    let (slope_y : UnreducedBigInt3) = unreduced_mul(slope, point.y)

    verify_zero(
        UnreducedBigInt3(
        d0=3 * x_sqr.d0 - 2 * slope_y.d0,
        d1=3 * x_sqr.d1 - 2 * slope_y.d1,
        d2=3 * x_sqr.d2 - 2 * slope_y.d2),
    )

    return (slope=slope)
end


func main{range_check_ptr: felt}():

    let x = BigInt3(1, 5, 10)
    let y = BigInt3(2, 4, 20)

    let point_a = EcPoint(x, y)
    let (point_b) = ec_negate(point_a)

    #compute_doubling_slope
    let (slope_a) = compute_doubling_slope(point_b)
    assert slope_a = BigInt3(64662730981121038053136098,32845645948216066767036314, 8201186782676455849150319)

    let (slope_b) = compute_doubling_slope(EcPoint(BigInt3(-1231, -51235643, -100000), BigInt3(77371252455, 7737125245, 19342813113)))
    assert slope_b = BigInt3(33416489251043008849460372,4045868738249434151710245, 18495428769257823271538303)

    return()
end

