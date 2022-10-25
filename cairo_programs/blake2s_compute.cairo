%builtins range_check bitwise

from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.memcpy import memcpy
from starkware.cairo.common.cairo_blake2s.blake2s import (
    INSTANCE_SIZE,
    STATE_SIZE_FELTS, 
    INPUT_BLOCK_FELTS,
    INPUT_BLOCK_BYTES, 
    blake2s_last_block,
    blake2s_add_felts,
    blake2s_add_uint256,
    blake2s_add_uint256_bigend,
    blake2s_bigend,
    blake2s_felts,
    finalize_blake2s,
)
from starkware.cairo.common.cairo_builtins import BitwiseBuiltin
from starkware.cairo.common.math_cmp import is_le
from starkware.cairo.common.uint256 import Uint256

func blake2s{range_check_ptr, blake2s_ptr : felt*}(data : felt*, n_bytes : felt) -> (res : Uint256):
    let (output) = blake2s_as_words(data=data, n_bytes=n_bytes)
    let res_low = output[3] * 2 ** 96 + output[2] * 2 ** 64 + output[1] * 2 ** 32 + output[0]
    let res_high = output[7] * 2 ** 96 + output[6] * 2 ** 64 + output[5] * 2 ** 32 + output[4]
    return (res=Uint256(low=res_low, high=res_high))
end

func blake2s_as_words{range_check_ptr, blake2s_ptr : felt*}(data : felt*, n_bytes : felt) -> (
    output : felt*
):
    assert blake2s_ptr[0] = 0x6B08E647
    assert blake2s_ptr[1] = 0xBB67AE85
    assert blake2s_ptr[2] = 0x3C6EF372
    assert blake2s_ptr[3] = 0xA54FF53A
    assert blake2s_ptr[4] = 0x510E527F
    assert blake2s_ptr[5] = 0x9B05688C
    assert blake2s_ptr[6] = 0x1F83D9AB
    assert blake2s_ptr[7] = 0x5BE0CD19
    static_assert STATE_SIZE_FELTS == 8
    let blake2s_ptr = blake2s_ptr + STATE_SIZE_FELTS

    let (output) = blake2s_inner(data=data, n_bytes=n_bytes, counter=0)
    return (output)
end

func blake2s_inner{range_check_ptr, blake2s_ptr : felt*}(
    data : felt*, n_bytes : felt, counter : felt
) -> (output : felt*):
    alloc_locals
    let (is_last_block) = is_le(n_bytes, INPUT_BLOCK_BYTES)
    if is_last_block != 0:
        return blake2s_last_block(data=data, n_bytes=n_bytes, counter=counter)
    end

    memcpy(blake2s_ptr, data, INPUT_BLOCK_FELTS)
    let blake2s_ptr = blake2s_ptr + INPUT_BLOCK_FELTS

    assert blake2s_ptr[0] = counter + INPUT_BLOCK_BYTES
    assert blake2s_ptr[1] = 0
    let blake2s_ptr = blake2s_ptr + 2

    let output = blake2s_ptr
    %{
        #TEST
        from starkware.cairo.common.cairo_blake2s.blake2s_utils import compute_blake2s_func
        compute_blake2s_func(segments=segments, output_ptr=ids.output)
        sleep(5)
        print("***HERE***")
    %}
    let blake2s_ptr = blake2s_ptr + STATE_SIZE_FELTS

    memcpy(blake2s_ptr, output, STATE_SIZE_FELTS)
    let blake2s_ptr = blake2s_ptr + STATE_SIZE_FELTS
    return blake2s_inner(
        data=data + INPUT_BLOCK_FELTS,
        n_bytes=n_bytes - INPUT_BLOCK_BYTES,
        counter=counter + INPUT_BLOCK_BYTES,
    )
end

func run_blake2s{range_check_ptr, blake2s_ptr : felt*}(input : felt*, lengths : felt*, n : felt):
    if n == 0:
        return ()
    end

    blake2s(input, lengths[0])
    return run_blake2s(input + 1, lengths + 1, n - 1)
end

func run_blake2s_and_finalize{range_check_ptr, bitwise_ptr : BitwiseBuiltin*}(
    input : felt*, lengths : felt*, n : felt
):
    alloc_locals
    let (local blake2s_ptr_start) = alloc()
    let blake2s_ptr = blake2s_ptr_start

    run_blake2s{blake2s_ptr=blake2s_ptr}(input, lengths, n)
    finalize_blake2s(blake2s_ptr_start=blake2s_ptr_start, blake2s_ptr_end=blake2s_ptr)
    return ()
end

func main{range_check_ptr, bitwise_ptr : BitwiseBuiltin*}():
    let (input) = alloc()
    let (lengths) = alloc()
    assert input[0] = 16
    assert lengths[0] = 1
    let n = 1
    run_blake2s_and_finalize(input, lengths, n)
    return()
end
