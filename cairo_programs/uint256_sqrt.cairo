from starkware.cairo.common.uint256 import Uint256, uint256_mul, uint256_le, uint256_sub
from uint256_add import uint256_add

# Returns the floor value of the square root of a uint256 integer.
func uint256_sqrt{range_check_ptr}(n: Uint256) -> (res: Uint256):
    alloc_locals
    local root: Uint256

    %{
        #TEST
        from starkware.python.math_utils import isqrt
        n = (ids.n.high << 128) + ids.n.low
        root = isqrt(n)
        assert 0 <= root < 2 ** 128
        ids.root.low = root
        ids.root.high = 0
    %}

    # Verify that 0 <= root < 2**128.
    assert root.high = 0
    [range_check_ptr] = root.low
    let range_check_ptr = range_check_ptr + 1

    # Verify that n >= root**2.
    let (root_squared, carry) = uint256_mul(root, root)
    assert carry = Uint256(0, 0)
    let (check_lower_bound) = uint256_le(root_squared, n)
    assert check_lower_bound = 1

    # Verify that n <= (root+1)**2 - 1.
    # In the case where root = 2**128 - 1, we will have next_root_squared=0.
    # Since (root+1)**2 = 2**256. Therefore next_root_squared - 1 = 2**256 - 1, as desired.
    let (next_root, add_carry) = uint256_add(root, Uint256(1, 0))
    assert add_carry = 0
    let (next_root_squared, _) = uint256_mul(next_root, next_root)
    let (next_root_squared_minus_one) = uint256_sub(next_root_squared, Uint256(1, 0))
    let (check_upper_bound) = uint256_le(n, next_root_squared_minus_one)
    assert check_upper_bound = 1

    return (res=root)
end

func main{range_check_ptr}():
    let a = Uint256(4,0)
    let (r) = uint256_sqrt(a)
    assert r = Uint256(2,0)
    return()
end
