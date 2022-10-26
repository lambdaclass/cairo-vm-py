%builtins range_check
from starkware.cairo.common.cairo_secp.bigint import BigInt3
from nondet_bigint3 import nondet_bigint3
from starkware.cairo.common.cairo_secp.bigint import (
    BASE,
    bigint_mul
    )
from starkware.cairo.common.cairo_secp.constants import BETA, N0, N1, N2


func div_mod_n{range_check_ptr}(a : BigInt3, b : BigInt3) -> (res : BigInt3):
    %{
        # TEST
        from starkware.cairo.common.cairo_secp.secp_utils import N, pack
        from starkware.python.math_utils import div_mod, safe_div

        a = pack(ids.a, PRIME)
        b = pack(ids.b, PRIME)
        value = res = div_mod(a, b, N)
    %}
    let (res) = nondet_bigint3()

    %{ 
        # TEST
        value = k = safe_div(res * b - a, N) 
    %}

    let (k) = nondet_bigint3()

    let (res_b) = bigint_mul(res, b)
    let n = BigInt3(N0, N1, N2)
    let (k_n) = bigint_mul(k, n)

    # We should now have res_b = k_n + a. Since the numbers are in unreduced form,
    # we should handle the carry.

    tempvar carry1 = (res_b.d0 - k_n.d0 - a.d0) / BASE
    assert [range_check_ptr + 0] = carry1 + 2 ** 127

    tempvar carry2 = (res_b.d1 - k_n.d1 - a.d1 + carry1) / BASE
    assert [range_check_ptr + 1] = carry2 + 2 ** 127

    tempvar carry3 = (res_b.d2 - k_n.d2 - a.d2 + carry2) / BASE
    assert [range_check_ptr + 2] = carry3 + 2 ** 127

    tempvar carry4 = (res_b.d3 - k_n.d3 + carry3) / BASE
    assert [range_check_ptr + 3] = carry4 + 2 ** 127

    assert res_b.d4 - k_n.d4 + carry4 = 0

    let range_check_ptr = range_check_ptr + 4

    return (res=res)
end

func main{range_check_ptr: felt} ():
    let a: BigInt3 = BigInt3(100,99,98)
    let b: BigInt3 = BigInt3(10,9,8)
    let (res) = div_mod_n(a, b)
    assert res.d0 = 3413472211745629263979533
    assert res.d1 = 17305268010345238170172332
    assert res.d2 = 11991751872105858217578135

    return()
end
