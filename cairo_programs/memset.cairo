# Writes value into [dst + 0], ..., [dst + n - 1].
from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.math import assert_not_zero

func memset(dst : felt*, value : felt, n):
    struct LoopFrame:
        member dst : felt*
    end

    if n == 0:
        return ()
    end

    %{
    # TEST
    vm_enter_scope({'n': ids.n})
    %}
    tempvar frame = LoopFrame(dst=dst)

    loop:
    let frame = [cast(ap - LoopFrame.SIZE, LoopFrame*)]
    assert [frame.dst] = value

    let continue_loop = [ap]
    # Reserve space for continue_loop.
    let next_frame = cast(ap + 1, LoopFrame*)
    next_frame.dst = frame.dst + 1; ap++
    %{
        # TEST
        n -= 1
        ids.continue_loop = 1 if n > 0 else 0
    %}
    static_assert next_frame + LoopFrame.SIZE == ap + 1
    jmp loop if continue_loop != 0; ap++
    # Assert that the loop executed n times.
    n = cast(next_frame.dst, felt) - cast(dst, felt)

    %{
    # TEST
    vm_exit_scope()
    %}
    return ()
end

func main():
    alloc_locals

    let dest: felt* = alloc()
    let value: felt = 1
    let n = 3

    memset(dest, value, n)

    assert_not_zero(dest[0])
    assert dest[0] = value
    assert_not_zero(dest[1])
    assert dest[1] = value
    assert_not_zero(dest[2])
    assert dest[2] = value
    return()
end

