%builtins range_check
from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.math import assert_nn_le

func find_element{range_check_ptr}(array_ptr : felt*, elm_size, n_elms, key) -> (elm_ptr : felt*):
    alloc_locals
    local index
    %{
        # TEST
        array_ptr = ids.array_ptr
        elm_size = ids.elm_size
        assert isinstance(elm_size, int) and elm_size > 0, \
            f'Invalid value for elm_size. Got: {elm_size}.'
        key = ids.key

        if '__find_element_index' in globals():
            ids.index = __find_element_index
            found_key = memory[array_ptr + elm_size * __find_element_index]
            assert found_key == key, \
                f'Invalid index found in __find_element_index. index: {__find_element_index}, ' \
                f'expected key {key}, found key: {found_key}.'
            # Delete __find_element_index to make sure it's not used for the next calls.
            del __find_element_index
        else:
            n_elms = ids.n_elms
            assert isinstance(n_elms, int) and n_elms >= 0, \
                f'Invalid value for n_elms. Got: {n_elms}.'
            if '__find_element_max_size' in globals():
                assert n_elms <= __find_element_max_size, \
                    f'find_element() can only be used with n_elms<={__find_element_max_size}. ' \
                    f'Got: n_elms={n_elms}.'

            for i in range(n_elms):
                if memory[array_ptr + elm_size * i] == key:
                    ids.index = i
                    break
            else:
                raise ValueError(f'Key {key} was not found.')
    %}

    assert_nn_le(a=index, b=n_elms - 1)
    tempvar elm_ptr = array_ptr + elm_size * index
    assert [elm_ptr] = key
    return (elm_ptr=elm_ptr)
end

struct MyStruct:
    member a : felt
    member b : felt
end

func main{range_check_ptr}() -> ():
    # Create an array with MyStruct elements (1,2), (3,4), (5,6).
    alloc_locals
    let (local array_ptr : MyStruct*) = alloc()
    assert array_ptr[0] = MyStruct(a=1, b=2)
    assert array_ptr[1] = MyStruct(a=3, b=4)
    assert array_ptr[2] = MyStruct(a=5, b=6)

    # Find any element with key '5'.
    let (element_ptr : MyStruct*) = find_element(
        array_ptr=array_ptr,
        elm_size=MyStruct.SIZE,
        n_elms=3,
        key=5,
    )
    # A pointer to the element with index 2 is returned.
    assert element_ptr.a = 5
    assert element_ptr.b = 6

    return ()
end
