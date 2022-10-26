%builtins range_check
from starkware.cairo.common.cairo_secp.bigint import (
    BigInt3,
    UnreducedBigInt3)
from nondet_bigint3 import nondet_bigint3
from starkware.cairo.common.cairo_secp.field import (
    unreduced_mul,
    verify_zero,
    unreduced_sqr,
)
from starkware.cairo.common.cairo_secp.ec import (EcPoint, compute_doubling_slope)

func ec_double{range_check_ptr}(point : EcPoint) -> (res : EcPoint):
    # The zero point.
    if point.x.d0 == 0:
        if point.x.d1 == 0:
            if point.x.d2 == 0:
                return (point)
            end
        end
    end

    let (slope : BigInt3) = compute_doubling_slope(point)
    let (slope_sqr : UnreducedBigInt3) = unreduced_sqr(slope)

    %{
        # TEST
        from starkware.cairo.common.cairo_secp.secp_utils import SECP_P, pack

        slope = pack(ids.slope, PRIME)
        x = pack(ids.point.x, PRIME)
        y = pack(ids.point.y, PRIME)

        value = new_x = (pow(slope, 2, SECP_P) - 2 * x) % SECP_P
    %}
    let (new_x : BigInt3) = nondet_bigint3()

    %{ 
        # TEST
        value = new_y = (slope * (x - new_x) - y) % SECP_P
    %}
    let (new_y : BigInt3) = nondet_bigint3()

    verify_zero(
        UnreducedBigInt3(
        d0=slope_sqr.d0 - new_x.d0 - 2 * point.x.d0,
        d1=slope_sqr.d1 - new_x.d1 - 2 * point.x.d1,
        d2=slope_sqr.d2 - new_x.d2 - 2 * point.x.d2),
    )

    let (x_diff_slope : UnreducedBigInt3) = unreduced_mul(
        BigInt3(d0=point.x.d0 - new_x.d0, d1=point.x.d1 - new_x.d1, d2=point.x.d2 - new_x.d2), slope
    )

    verify_zero(
        UnreducedBigInt3(
        d0=x_diff_slope.d0 - point.y.d0 - new_y.d0,
        d1=x_diff_slope.d1 - point.y.d1 - new_y.d1,
        d2=x_diff_slope.d2 - point.y.d2 - new_y.d2),
    )

    return (res=EcPoint(new_x, new_y))
end

func main{range_check_ptr: felt}():

    let x = BigInt3(1, 5, 10)
    let y = BigInt3(2, 4, 20)
    let point_a = EcPoint(x, y)

    #ec_double
    let (point_d) = ec_double(point_a)
    assert point_d = EcPoint(BigInt3(74427550641062819382893486, 40869730155367266160799328, 5674783931833640986577252), BigInt3(30795856170124638149720790, 54408100978340609265106444, 13350501717657408140240292))

    let (point_e) = ec_double(EcPoint(BigInt3(156, 6545, 100010), BigInt3(-5336262886225868, 1324, -113834066795297906)))
    assert point_e = EcPoint(BigInt3(55117564152931927789817182, 33048130247267262167865975, 14533608608654363688616034), BigInt3(54056253314096377704781816, 68158355584365770862343034, 3052322168655618600739346))
    return ()
end
