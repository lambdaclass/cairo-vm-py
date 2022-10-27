%builtins range_check

from starkware.cairo.common.cairo_secp.bigint import BigInt3, UnreducedBigInt3
from starkware.cairo.common.cairo_secp.constants import BASE
from starkware.cairo.common.cairo_secp.field import verify_zero
from nondet_bigint3 import nondet_bigint3

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
