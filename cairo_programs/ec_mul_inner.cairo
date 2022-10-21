%builtins range_check

from starkware.cairo.common.cairo_secp.ec import (
        EcPoint,
        ec_negate,
        compute_doubling_slope,
        compute_slope,
        ec_double,
        fast_ec_add,
)
from starkware.cairo.common.cairo_secp.bigint import BigInt3

func ec_mul_inner{range_check_ptr}(point : EcPoint, scalar : felt, m : felt) -> (
    pow2 : EcPoint, res : EcPoint
):
    if m == 0:
        with_attr error_message("Too large scalar"):
            scalar = 0
        end
        let ZERO_POINT = EcPoint(BigInt3(0, 0, 0), BigInt3(0, 0, 0))
        return (pow2=point, res=ZERO_POINT)
    end

    alloc_locals
    let (double_point : EcPoint) = ec_double(point)
    %{ 
    #TEST
    memory[ap] = (ids.scalar % PRIME) % 2
    %}
    jmp odd if [ap] != 0; ap++
    return ec_mul_inner(point=double_point, scalar=scalar / 2, m=m - 1)

    odd:
    let (local inner_pow2 : EcPoint, inner_res : EcPoint) = ec_mul_inner(
        point=double_point, scalar=(scalar - 1) / 2, m=m - 1
    )
    # Here inner_res = (scalar - 1) / 2 * double_point = (scalar - 1) * point.
    # Assume point != 0 and that inner_res = +/-point. We obtain (scalar - 1) * point = +/-point =>
    # scalar - 1 = +/-1 (mod N) => scalar = 0 or 2 (mod N).
    # By induction, we know that (scalar - 1) / 2 must be in the range [0, 2**(m-1)),
    # so scalar is an odd number in the range [0, 2**m), and we get a contradiction.
    let (res : EcPoint) = fast_ec_add(point0=point, point1=inner_res)
    return (pow2=inner_pow2, res=res)
end

func main{range_check_ptr: felt}():
    #ec_mul_inner
    let (pow2, res) = ec_mul_inner(EcPoint(BigInt3(65162296, 359657, 04862662171381), BigInt3(-5166641367474701, -63029418, 793)), 123, 298)
    assert pow2 = EcPoint(BigInt3(30016796425722798916160189, 75045389156830800234717485, 13862403786096360935413684), BigInt3(43820690643633544357415586, 29808113745001228006676979, 15112469502208690731782390))
    return()
end
