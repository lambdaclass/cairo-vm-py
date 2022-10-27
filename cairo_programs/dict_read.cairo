from starkware.cairo.common.dict_access import DictAccess
from dict_new import dict_new

func dict_read{dict_ptr : DictAccess*}(key : felt) -> (value : felt):
    alloc_locals
    local value
    %{
        #TEST
        dict_tracker = __dict_manager.get_tracker(ids.dict_ptr)
        dict_tracker.current_ptr += ids.DictAccess.SIZE
        ids.value = dict_tracker.data[ids.key]
    %}
    assert dict_ptr.key = key
    assert dict_ptr.prev_value = value
    assert dict_ptr.new_value = value
    let dict_ptr = dict_ptr + DictAccess.SIZE
    return (value=value)
end

func main():
    alloc_locals
    %{ initial_dict = {1:2, 2:3, 4:5}%}
    let (my_dict) = dict_new()
    let (local val1 : felt) = dict_read{dict_ptr=my_dict}(key=1)
    assert val1 = 2
    let (local val2 : felt) = dict_read{dict_ptr=my_dict}(key=2)
    assert val2 = 3
    let (local val3 : felt) = dict_read{dict_ptr=my_dict}(key=4)
    assert val3 = 5
    return ()
end
