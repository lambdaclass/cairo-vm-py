%builtins range_check bitwise

from starkware.cairo.common.alloc import alloc
from blake2s_module import blake2s, finalize_blake2s, blake2s_felts
from starkware.cairo.common.cairo_builtins import BitwiseBuiltin
from starkware.cairo.common.uint256 import Uint256
from starkware.cairo.common.bool import TRUE, FALSE

func fill_array(array: felt*, base: felt, step: felt, array_length: felt, iterator: felt):
    if iterator == array_length:
        return()
    end
    assert array[iterator] = base + step * iterator
    return fill_array(array, base, step, array_length, iterator + 1)
end

func test_integration{range_check_ptr, bitwise_ptr : BitwiseBuiltin*}(iter : felt, last : felt):
    alloc_locals
    if iter == last:
        return ()
    end

    let (data : felt*) = alloc()
    fill_array(data, iter, 2*iter, 10, 0)

    let (local blake2s_ptr_start) = alloc()
    let blake2s_ptr = blake2s_ptr_start
    let (res_1 : Uint256) =  blake2s{range_check_ptr=range_check_ptr, blake2s_ptr=blake2s_ptr}(data, 9)

    finalize_blake2s(blake2s_ptr_start, blake2s_ptr)

    let (local blake2s_ptr_start) = alloc()
    let blake2s_ptr = blake2s_ptr_start

    let (data_2 : felt*) = alloc()
    assert data_2[0] = res_1.low
    assert data_2[1] = res_1.high

    let (res_2) =  blake2s_felts{range_check_ptr=range_check_ptr, blake2s_ptr=blake2s_ptr}(2, data_2, FALSE)

    finalize_blake2s(blake2s_ptr_start, blake2s_ptr)

    if iter == last - 1 and last == 5:
        assert res_1.low = 325391546354419665382867554662450050592
        assert res_1.high = 730650687217386792084071539158056142
        assert res_2.low = 126060886735271509991955977259939811963
        assert res_2.high = 122931524979943397295832144754045746270
    end

    return test_integration(iter+1, last)
end

func run_tests{range_check_ptr, bitwise_ptr : BitwiseBuiltin*}(last : felt):
    alloc_locals
    test_integration(0, last)

    return ()
end

func main{range_check_ptr, bitwise_ptr : BitwiseBuiltin*}():
    run_tests(5)

    return ()
end
