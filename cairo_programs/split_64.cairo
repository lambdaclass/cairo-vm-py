%builtins range_check

from starkware.cairo.common.uint256 import HALF_SHIFT

func split_64{range_check_ptr}(a : felt) -> (low : felt, high : felt):
    alloc_locals
    local low : felt
    local high : felt

    %{
        #TEST
        ids.low = ids.a & ((1<<64) - 1)
        ids.high = ids.a >> 64
    %}
    assert a = low + high * HALF_SHIFT
    assert [range_check_ptr + 0] = low
    assert [range_check_ptr + 1] = HALF_SHIFT - 1 - low
    assert [range_check_ptr + 2] = high
    let range_check_ptr = range_check_ptr + 3
    return (low, high)
end

func main{range_check_ptr: felt}():
    let (low, high) = split_64(850981239023189021389081239089023)
    assert low = 7249717543555297151
    assert high = 46131785404667
    return()
end
