from starkware.cairo.common.dict_access import DictAccess
from starkware.cairo.common.cairo_secp.bigint import BigInt3

struct UnreducedBigInt3 {
    d0: felt,
    d1: felt,
    d2: felt,
}

func check_sizes() {
    %{
        #TEST
        assert(3 == ids.DictAccess.SIZE)
        assert(3 == ids.BigInt3.SIZE)
        assert(3 == ids.UnreducedBigInt3.SIZE)
        #assert(4 == ids.Four.SIZE)
    %}
    return ();
}

func main() {
    struct Four {
        d0: felt,
        d1: felt,
        d2: felt,
        d3: felt,
    }
    check_sizes();
    return ();
}
