%builtins range_check
from starkware.cairo.common.math import assert_nn_le
from starkware.cairo.common.memcpy import memcpy


from starkware.cairo.common.alloc import alloc

struct MyStruct:
    member a : felt
    member b : felt
end

func set_add{range_check_ptr, set_end_ptr : felt*}(set_ptr : felt*, elm_size, elm_ptr : felt*):
    alloc_locals
    local is_elm_in_set
    local index
    %{
        # TEST
        assert ids.elm_size > 0
        assert ids.set_ptr <= ids.set_end_ptr
        elm_list = memory.get_range(ids.elm_ptr, ids.elm_size)
        for i in range(0, ids.set_end_ptr - ids.set_ptr, ids.elm_size):
            if memory.get_range(ids.set_ptr + i, ids.elm_size) == elm_list:
                ids.index = i // ids.elm_size
                ids.is_elm_in_set = 1
                break
        else:
            ids.is_elm_in_set = 0
    %}
    if is_elm_in_set != 0:
        local located_elm_ptr : felt* = set_ptr + elm_size * index
        # Using memcpy for equality assertion.
        memcpy(dst=located_elm_ptr, src=elm_ptr, len=elm_size)
        tempvar n_elms = (cast(set_end_ptr, felt) - cast(set_ptr, felt)) / elm_size
        assert_nn_le(index, n_elms - 1)
        return ()
    else:
        memcpy(dst=set_end_ptr, src=elm_ptr, len=elm_size)
        let set_end_ptr : felt* = set_end_ptr + elm_size
        return ()
    end
end


func main{range_check_ptr}():
    alloc_locals

    # An array containing two structs.
    let (local my_list : MyStruct*) = alloc()
    assert my_list[0] = MyStruct(a=1, b=3)
    assert my_list[1] = MyStruct(a=5, b=7)

    # Suppose that we want to add the element
    # MyStruct(a=2, b=3) to my_list, but only if it is not already
    # present (for the purpose of the example the contents of the
    # array are known, but this doesn't have to be the case)
    let list_end : felt* = &my_list[2]
    let (new_elm : MyStruct*) = alloc()
    assert new_elm[0] = MyStruct(a=2, b=3)

    set_add{set_end_ptr=list_end}(
        set_ptr=my_list, elm_size=MyStruct.SIZE, elm_ptr=new_elm
    )
    assert my_list[2] = MyStruct(a=2, b=3)

    # Now let's try to add MyStruct(a=1, b=3) to my_list,
    # Since this element is already present in my_list,
    # set_add won't add any element to the my_list

    let list_end : felt* = &my_list[3]
    assert new_elm[1] = MyStruct(a=1, b=3)

    set_add{set_end_ptr=list_end}(
        set_ptr=my_list, elm_size=MyStruct.SIZE, elm_ptr=new_elm
    )
    
    # Since my_list[3] = None, we can insert a MyStruct
    assert my_list[3] = MyStruct(a=0, b=0)

    return ()
end
