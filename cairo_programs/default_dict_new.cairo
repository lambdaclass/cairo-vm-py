from starkware.cairo.common.dict_access import DictAccess

# Creates a new dictionary, with a default value.
func default_dict_new(default_value : felt) -> (res : DictAccess*):
    %{
        #TEST
        if '__dict_manager' not in globals():
            from starkware.cairo.common.dict import DictManager
            __dict_manager = DictManager()

        memory[ap] = __dict_manager.new_default_dict(segments, ids.default_value)
    %}
    ap += 1
    return (res=cast([ap - 1], DictAccess*))
end

func main():
    let my_dict = default_dict_new(17)
    return ()
end
