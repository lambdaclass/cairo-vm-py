from starkware.cairo.common.dict_access import DictAccess
from starkware.cairo.common.cairo_secp.bigint import BigInt3

struct UnreducedBigInt3:
    member d0 : felt
    member d1 : felt
    member d2 : felt
end

func check_sizes():
    %{
        #TEST
        assert(3 == ids.DictAccess.SIZE)
        assert(3 == ids.BigInt3.SIZE)
        assert(3 == ids.UnreducedBigInt3.SIZE)
        #assert(4 == ids.Four.SIZE)
    %}
    return ()
end

func main():
    struct Four:
        member d0 : felt
        member d1 : felt
        member d2 : felt
        member d3 : felt
    end
    check_sizes()
    return ()
end
