%builtins range_check
from starkware.cairo.common.cairo_secp.bigint import (
    BigInt3,
    UnreducedBigInt3)
from nondet_bigint3 import nondet_bigint3
from starkware.cairo.common.cairo_secp.field import (
    reduce,
    unreduced_mul,
    unreduced_sqr,
    verify_zero,
)
from starkware.cairo.common.cairo_secp.ec import EcPoint
from starkware.cairo.common.math import assert_nn
from starkware.cairo.common.cairo_secp.constants import BETA

func get_point_from_x{range_check_ptr}(x : BigInt3, v : felt) -> (point : EcPoint):
    with_attr error_message("Out of range v {v}."):
        assert_nn(v)
    end
    let (x_square : UnreducedBigInt3) = unreduced_sqr(x)
    let (x_square_reduced : BigInt3) = reduce(x_square)
    let (x_cube : UnreducedBigInt3) = unreduced_mul(x, x_square_reduced)

    %{
        # TEST
        from starkware.cairo.common.cairo_secp.secp_utils import SECP_P, pack

        x_cube_int = pack(ids.x_cube, PRIME) % SECP_P
        y_square_int = (x_cube_int + ids.BETA) % SECP_P
        y = pow(y_square_int, (SECP_P + 1) // 4, SECP_P)

        # We need to decide whether to take y or SECP_P - y.
        if ids.v % 2 == y % 2:
            value = y
        else:
            value = (-y) % SECP_P
    %}
    let (y : BigInt3) = nondet_bigint3()

    # Check that y has same parity as v.
    assert_nn((y.d0 + v) / 2)

    let (y_square : UnreducedBigInt3) = unreduced_sqr(y)
    # Check that y_square = x_cube + BETA.
    verify_zero(
        UnreducedBigInt3(
        d0=x_cube.d0 + BETA - y_square.d0,
        d1=x_cube.d1 - y_square.d1,
        d2=x_cube.d2 - y_square.d2,
        ),
    )

    return (point=EcPoint(x, y))
end

func main{range_check_ptr: felt} ():

    let x: BigInt3 = BigInt3(100,99,98)
    let v: felt = 10
    let (point) = get_point_from_x(x, v)
    assert point.x.d0 = 100
    assert point.x.d1 = 99
    assert point.x.d2 = 98
    assert point.y.d0 = 50471654703173585387369794
    assert point.y.d1 = 68898944762041070370364387
    assert point.y.d2 = 16932612780945290933872774
    return()
end
