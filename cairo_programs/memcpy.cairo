from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.math import assert_not_zero


# Copies len field elements from src to dst.
func memcpy(dst : felt*, src : felt*, len):
    struct LoopFrame:
        member dst : felt*
        member src : felt*
    end

    if len == 0:
        return ()
    end

    %{
    #TEST
    vm_enter_scope({'n': ids.len})
		%}
    tempvar frame = LoopFrame(dst=dst, src=src)

    loop:
    let frame = [cast(ap - LoopFrame.SIZE, LoopFrame*)]
    assert [frame.dst] = [frame.src]

    let continue_copying = [ap]
    # Reserve space for continue_copying.
    let next_frame = cast(ap + 1, LoopFrame*)
    next_frame.dst = frame.dst + 1; ap++
    next_frame.src = frame.src + 1; ap++
    %{
    #TEST
    n -= 1
    ids.continue_copying = 1 if n > 0 else 0
    %}
    static_assert next_frame + LoopFrame.SIZE == ap + 1
    jmp loop if continue_copying != 0; ap++
    # Assert that the loop executed len times.
    len = cast(next_frame.src, felt) - cast(src, felt)

    %{
    #TEST
    vm_exit_scope()
		%}
    return ()
end

func main():
	alloc_locals

	let dest: felt* = alloc()
	let source: felt* = alloc()

	assert source[0] = 1
	assert source[1] = 1
	assert source[2] = 1

	memcpy(dest, source, 3)

	assert_not_zero(dest[0])
  assert dest[0] = source[0]
  assert_not_zero(dest[1])
	assert dest[1] = source[1]
  assert_not_zero(dest[2])
	assert dest[2] = source[2]

	return()
end

