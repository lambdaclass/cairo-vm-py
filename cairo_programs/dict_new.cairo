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

func main():
    %{ initial_dict = {1:2, 2:3, 4:5}%}
    let my_dict = dict_new()
    return ()
end
