%builtins range_check

from starkware.cairo.common.cairo_secp.bigint import BigInt3, UnreducedBigInt3
from starkware.cairo.common.cairo_secp.constants import BASE
from starkware.cairo.common.cairo_secp.field import verify_zero


func nondet_bigint3{range_check_ptr}() -> (res : BigInt3):
    # The result should be at the end of the stack after the function returns.
    let res : BigInt3 = [cast(ap + 5, BigInt3*)]
    %{
        # TEST
        from starkware.cairo.common.cairo_secp.secp_utils import split

        segments.write_arg(ids.res.address_, split(value))
    %}
    # The maximal possible sum of the limbs, assuming each of them is in the range [0, BASE).
    const MAX_SUM = 3 * (BASE - 1)
    assert [range_check_ptr] = MAX_SUM - (res.d0 + res.d1 + res.d2)

    # Prepare the result at the end of the stack.
    tempvar range_check_ptr = range_check_ptr + 4
    [range_check_ptr - 3] = res.d0; ap++
    [range_check_ptr - 2] = res.d1; ap++
    [range_check_ptr - 1] = res.d2; ap++
    static_assert &res + BigInt3.SIZE == ap
    return (res=res)
end

func reduce{range_check_ptr}(x : UnreducedBigInt3) -> (reduced_x : BigInt3):
    %{
        # TEST
        from starkware.cairo.common.cairo_secp.secp_utils import SECP_P, pack

        value = pack(ids.x, PRIME) % SECP_P

    %}
    let (reduced_x : BigInt3) = nondet_bigint3()

    verify_zero(
        UnreducedBigInt3(
        d0=x.d0 - reduced_x.d0,
        d1=x.d1 - reduced_x.d1,
        d2=x.d2 - reduced_x.d2),
    )
    return (reduced_x=reduced_x)
end


func main{range_check_ptr}():
    let n: BigInt3 = reduce(UnreducedBigInt3(1321812083892150,11230,103321))
    assert n = BigInt3(1321812083892150,11230,103321)    
    return ()
end
