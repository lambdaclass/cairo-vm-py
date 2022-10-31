%builtins range_check
from starkware.cairo.common.math import assert_nn_le
from starkware.cairo.common.alloc import alloc

func split_int{range_check_ptr}(value, n, base, bound, output : felt*):
    if n == 0:
        %{ assert ids.value == 0, 'split_int(): value is out of range.' #TEST %}
        assert value = 0
        return ()
    end

    %{
        #TEST
        memory[ids.output] = res = (int(ids.value) % PRIME) % ids.base
        assert res < ids.bound, f'split_int(): Limb {res} is out of range.'
    %}
    tempvar low_part = [output]
    assert_nn_le(low_part, bound - 1)

    return split_int(
        value=(value - low_part) / base, n=n - 1, base=base, bound=bound, output=output + 1
    )
end

func main{range_check_ptr: felt}():
    alloc_locals
    let value = 3618502788666131213697322783095070105623117215331596699973092056135872020481
    let n = 2
    let base = 2**64
    let bound = 2**64
    let output: felt* = alloc()
    split_int(value, n, base, bound, output)
    assert output[0] = 4003012203950112768
    assert output[1] = 542101086242752
    return()
end
