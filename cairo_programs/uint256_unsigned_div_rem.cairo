from starkware.cairo.common.uint256 import Uint256, uint256_mul, uint256_lt
from uint256_add import uint256_add

# Unsigned integer division between two integers. Returns the quotient and the remainder.
# Conforms to EVM specifications: division by 0 yields 0.
func uint256_unsigned_div_rem{range_check_ptr}(a : Uint256, div : Uint256) -> (
        quotient : Uint256, remainder : Uint256):
    alloc_locals
    local quotient : Uint256
    local remainder : Uint256

    # If div == 0, return (0, 0).
    if div.low + div.high == 0:
        return (quotient=Uint256(0, 0), remainder=Uint256(0, 0))
    end

    %{
        #TEST
        a = (ids.a.high << 128) + ids.a.low
        div = (ids.div.high << 128) + ids.div.low
        quotient, remainder = divmod(a, div)

        ids.quotient.low = quotient & ((1 << 128) - 1)
        ids.quotient.high = quotient >> 128
        ids.remainder.low = remainder & ((1 << 128) - 1)
        ids.remainder.high = remainder >> 128
    %}
    let (res_mul, carry) = uint256_mul(quotient, div)
    assert carry = Uint256(0, 0)

    let (check_val, add_carry) = uint256_add(res_mul, remainder)
    assert check_val = a
    assert add_carry = 0

    let (is_valid) = uint256_lt(remainder, div)
    assert is_valid = 1
    return (quotient=quotient, remainder=remainder)
end

func main{range_check_ptr}():
    let a = Uint256(4,0)
    let b = Uint256(2,0)
    let (r, c) = uint256_unsigned_div_rem(a, b)
    assert r = Uint256(2,0)
    assert c = Uint256(0,0)
    return()
end
