%builtins range_check
from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.math import assert_nn_le, assert_le_felt


func search_sorted_lower{range_check_ptr}(array_ptr : felt*, elm_size, n_elms, key) -> (
    elm_ptr : felt*
):
    alloc_locals
    local index
    %{
        # TEST
        array_ptr = ids.array_ptr
        elm_size = ids.elm_size
        assert isinstance(elm_size, int) and elm_size > 0, \
            f'Invalid value for elm_size. Got: {elm_size}.'

        n_elms = ids.n_elms
        assert isinstance(n_elms, int) and n_elms >= 0, \
            f'Invalid value for n_elms. Got: {n_elms}.'
        if '__find_element_max_size' in globals():
            assert n_elms <= __find_element_max_size, \
                f'find_element() can only be used with n_elms<={__find_element_max_size}. ' \
                f'Got: n_elms={n_elms}.'

        for i in range(n_elms):
            if memory[array_ptr + elm_size * i] >= ids.key:
                ids.index = i
                break
        else:
            ids.index = n_elms
    %}

    assert_nn_le(a=index, b=n_elms)
    local elm_ptr : felt* = array_ptr + elm_size * index

    if index != n_elms:
        assert_le_felt(a=key, b=[elm_ptr])
    else:
        tempvar range_check_ptr = range_check_ptr
    end

    if index != 0:
        assert_le_felt(a=[elm_ptr - elm_size] + 1, b=key)
    end

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
    let (smallest_ptr : MyStruct*) = search_sorted_lower(
        array_ptr=array_ptr, elm_size=2, n_elms=3, key=2
    )
    assert smallest_ptr.a = 3
    assert smallest_ptr.b = 4
    return ()
end
