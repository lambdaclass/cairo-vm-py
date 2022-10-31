%builtins range_check
from starkware.cairo.common.uint256 import Uint256

# Returns 1 if the signed integer is nonnegative.
func uint256_signed_nn{range_check_ptr}(a: Uint256) -> (res: felt):
    %{ 
        #TEST
        memory[ap] = 1 if 0 <= (ids.a.high % PRIME) < 2 ** 127 else 0 
    %}
    jmp non_negative if [ap] != 0; ap++

    assert [range_check_ptr] = a.high - 2 ** 127
    let range_check_ptr = range_check_ptr + 1
    return (res=0)

    non_negative:
    assert [range_check_ptr] = a.high + 2 ** 127
    let range_check_ptr = range_check_ptr + 1
    return (res=1)
end

func main{range_check_ptr}():
    let n = Uint256(1,0)
    let (r) = uint256_signed_nn(n)
    assert r = 1
    return()
end
