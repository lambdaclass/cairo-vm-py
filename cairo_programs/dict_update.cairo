from starkware.cairo.common.dict_access import DictAccess
from dict_new import dict_new
from dict_read import dict_read

# Updates a value in a dict. prev_value must be specified. A standalone read with no write should be
# performed by writing the same value.
# It is possible to get prev_value from __dict_manager using the hint:
#   %{ ids.val = __dict_manager.get_dict(ids.dict_ptr)[ids.key] %}
func dict_update{dict_ptr : DictAccess*}(key : felt, prev_value : felt, new_value : felt):
    %{
        #TEST
        # Verify dict pointer and prev value.
        dict_tracker = __dict_manager.get_tracker(ids.dict_ptr)
        current_value = dict_tracker.data[ids.key]
        assert current_value == ids.prev_value, \
            f'Wrong previous value in dict. Got {ids.prev_value}, expected {current_value}.'

        # Update value.
        dict_tracker.data[ids.key] = ids.new_value
        dict_tracker.current_ptr += ids.DictAccess.SIZE
    %}
    dict_ptr.key = key
    dict_ptr.prev_value = prev_value
    dict_ptr.new_value = new_value
    let dict_ptr = dict_ptr + DictAccess.SIZE
    return ()
end

func main():
    alloc_locals
    %{ initial_dict = {1:2, 2:3, 4:5}%}
    let (my_dict) = dict_new()
    dict_update{dict_ptr=my_dict}(key=1, prev_value=2, new_value=5)
    let (local val : felt) = dict_read{dict_ptr=my_dict}(key=1)
    assert val = 5
    return ()
end
