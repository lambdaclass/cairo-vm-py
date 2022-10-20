from starkware.cairo.common.dict_access import DictAccess

# Creates a new dict.
# Hint argument:
# initial_dict - A python dict containing the initial values of the new dict.
func dict_new() -> (res: DictAccess*):
    %{
        #TEST
        if '__dict_manager' not in globals():
            from starkware.cairo.common.dict import DictManager
            __dict_manager = DictManager()

        memory[ap] = __dict_manager.new_dict(segments, initial_dict)
        del initial_dict
    %}
    ap += 1
    return (res=cast([ap - 1], DictAccess*))
end

func dict_read{dict_ptr : DictAccess*}(key : felt) -> (value : felt):
    alloc_locals
    local value
    %{
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
    let my_dict = dict_new()
    let (local val1 : felt) = dict_read{dict_ptr=my_dict}(key=1)
    assert val1 = 2
    return ()
end
