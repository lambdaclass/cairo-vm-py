%builtins range_check
from nondet_bigint3 import nondet_bigint3
from starkware.cairo.common.cairo_secp.bigint import BigInt3, UnreducedBigInt3
from starkware.cairo.common.cairo_secp.field import is_zero, unreduced_mul, unreduced_sqr, verify_zero
from starkware.cairo.common.cairo_secp.ec import EcPoint, compute_slope, ec_double

func fast_ec_add{range_check_ptr}(point0 : EcPoint, point1 : EcPoint) -> (res : EcPoint):
    # Check whether point0 is the zero point.
    if point0.x.d0 == 0:
        if point0.x.d1 == 0:
            if point0.x.d2 == 0:
                return (point1)
            end
        end
    end

    # Check whether point1 is the zero point.
    if point1.x.d0 == 0:
        if point1.x.d1 == 0:
            if point1.x.d2 == 0:
                return (point0)
            end
        end
    end

    let (slope : BigInt3) = compute_slope(point0, point1)
    let (slope_sqr : UnreducedBigInt3) = unreduced_sqr(slope)

    %{
        # TEST
        from starkware.cairo.common.cairo_secp.secp_utils import SECP_P, pack

        slope = pack(ids.slope, PRIME)
        x0 = pack(ids.point0.x, PRIME)
        x1 = pack(ids.point1.x, PRIME)
        y0 = pack(ids.point0.y, PRIME)

        value = new_x = (pow(slope, 2, SECP_P) - x0 - x1) % SECP_P
    %}
    let (new_x : BigInt3) = nondet_bigint3()

    %{
    # TEST 
    value = new_y = (slope * (x0 - new_x) - y0) % SECP_P 
    %}

    let (new_y : BigInt3) = nondet_bigint3()

    verify_zero(
        UnreducedBigInt3(
        d0=slope_sqr.d0 - new_x.d0 - point0.x.d0 - point1.x.d0,
        d1=slope_sqr.d1 - new_x.d1 - point0.x.d1 - point1.x.d1,
        d2=slope_sqr.d2 - new_x.d2 - point0.x.d2 - point1.x.d2),
    )

    let (x_diff_slope : UnreducedBigInt3) = unreduced_mul(
        BigInt3(d0=point0.x.d0 - new_x.d0, d1=point0.x.d1 - new_x.d1, d2=point0.x.d2 - new_x.d2),
        slope,
    )

    verify_zero(
        UnreducedBigInt3(
        d0=x_diff_slope.d0 - point0.y.d0 - new_y.d0,
        d1=x_diff_slope.d1 - point0.y.d1 - new_y.d1,
        d2=x_diff_slope.d2 - point0.y.d2 - new_y.d2),
    )

    return (EcPoint(new_x, new_y))
end


func main{range_check_ptr: felt}():

    let x = BigInt3(1, 5, 10)
    let y = BigInt3(2, 4, 20)

    let point_a = EcPoint(x, y)
    let (point_e) = ec_double(EcPoint(BigInt3(156, 6545, 100010), BigInt3(-5336262886225868, 1324, -113834066795297906)))


    let (point_f) = fast_ec_add(point_a, point_e)
    assert point_f = EcPoint(BigInt3(69178603654448607465162296, 33667561357032241906559657, 11638763416304862662171381), BigInt3(51035566479066641367474701, 39483223302560035063029418, 12190232481429041491400793))
    
    let (point_g) = fast_ec_add(
        EcPoint(BigInt3(89712, 56, -109), BigInt3(980126, 10, 8793)),
        EcPoint(BigInt3(-16451, 5967, 2171381), BigInt3(-12364564, -123654, 193))
        )
    assert point_g = EcPoint(BigInt3(33668922213009861691786428, 29470240120447974127390849, 12360778067138644393307525), BigInt3(11020030022607540331466881, 148713025757531154701204, 8824915433273552029783507))

    return()
end
