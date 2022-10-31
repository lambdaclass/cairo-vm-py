from starkware.cairo.common.dict_access import DictAccess
from dict_new import dict_new
from dict_read import dict_read

# Writes a value to the dictionary, overriding the existing value.
func dict_write{dict_ptr : DictAccess*}(key : felt, new_value : felt):
    %{
        #TEST
        dict_tracker = __dict_manager.get_tracker(ids.dict_ptr)
        dict_tracker.current_ptr += ids.DictAccess.SIZE
        ids.dict_ptr.prev_value = dict_tracker.data[ids.key]
        dict_tracker.data[ids.key] = ids.new_value
    %}
    assert dict_ptr.key = key
    assert dict_ptr.new_value = new_value
    let dict_ptr = dict_ptr + DictAccess.SIZE
    return ()
end

func main():
    alloc_locals
    %{ initial_dict = {1:2, 2:3, 4:5}%}
    let (my_dict) = dict_new()
    dict_write{dict_ptr=my_dict}(key=1, new_value=34)
    let (local val : felt) = dict_read{dict_ptr=my_dict}(key=1)
    assert val = 34
    return ()
end
