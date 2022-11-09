%builtins range_check
from starkware.cairo.common.uint256 import uint256_check, Uint256

const SHIFT = 2 ** 128

# Adds two integers. Returns the result as a 256-bit integer and the (1-bit) carry.
func uint256_add{range_check_ptr}(a : Uint256, b : Uint256) -> (res : Uint256, carry : felt):
    alloc_locals
    local res : Uint256
    local carry_low : felt
    local carry_high : felt
    %{
        #TEST
        sum_low = ids.a.low + ids.b.low
        ids.carry_low = 1 if sum_low >= ids.SHIFT else 0
        sum_high = ids.a.high + ids.b.high + ids.carry_low
        ids.carry_high = 1 if sum_high >= ids.SHIFT else 0
    %}

    assert carry_low * carry_low = carry_low
    assert carry_high * carry_high = carry_high

    assert res.low = a.low + b.low - carry_low * SHIFT
    assert res.high = a.high + b.high + carry_low - carry_high * SHIFT
    uint256_check(res)

    return (res, carry_high)
end

func main{range_check_ptr}():
    let a = Uint256(1,0)
    let b = Uint256(2,0)
    let (r, c) = uint256_add(a, b)
    assert r = Uint256(3,0)
    assert c = 0
    return()
end
