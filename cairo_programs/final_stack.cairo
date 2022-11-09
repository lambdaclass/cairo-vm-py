%builtins range_check bitwise

from starkware.cairo.common.bitwise import bitwise_and, bitwise_xor, bitwise_or, bitwise_operations
from starkware.cairo.common.cairo_builtins import BitwiseBuiltin


func assert_le{range_check_ptr}(a, b):
    %{
        # TEST
        a = ids.a % PRIME
        b = ids.b % PRIME
        assert a <= b, f'a = {a} is not less than or equal to b = {b}.'
    %}
    return ()
end


func main{range_check_ptr, bitwise_ptr: BitwiseBuiltin*}():
    let (and_a) = bitwise_and(12, 10)  # Binary (1100, 1010).
    assert and_a = 8  # Binary 1000.
    let (xor_a) = bitwise_xor(12, 10)
    assert xor_a = 6
    let (or_a) = bitwise_or(12, 10)
    assert or_a = 14

    let (and_b, xor_b, or_b) = bitwise_operations(9, 3)
    assert and_b = 1
    assert xor_b = 10
    assert or_b = 11

    let a: felt = 1
    let b: felt = 2
    assert_le(a, b)
    return ()
end
