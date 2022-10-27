from starkware.cairo.common.dict_access import DictAccess
from dict_update import dict_update
from dict_new import dict_new
from squash_dict import squash_dict

func dict_squash{range_check_ptr}(
        dict_accesses_start : DictAccess*, dict_accesses_end : DictAccess*) -> (
        squashed_dict_start : DictAccess*, squashed_dict_end : DictAccess*):
    alloc_locals

    %{
        #TEST
        # Prepare arguments for dict_new. In particular, the same dictionary values should be copied
        # to the new (squashed) dictionary.
        vm_enter_scope({
            # Make __dict_manager accessible.
            '__dict_manager': __dict_manager,
            # Create a copy of the dict, in case it changes in the future.
            'initial_dict': dict(__dict_manager.get_dict(ids.dict_accesses_end)),
        })
    %}
    let (local squashed_dict_start) = dict_new()
    %{ 
        #TEST
        vm_exit_scope() 
    %}

    let (squashed_dict_end) = squash_dict(
        dict_accesses=dict_accesses_start,
        dict_accesses_end=dict_accesses_end,
        squashed_dict=squashed_dict_start)

    %{
        #TEST
        # Update the DictTracker's current_ptr to point to the end of the squashed dict.
        __dict_manager.get_tracker(ids.squashed_dict_start).current_ptr = \
            ids.squashed_dict_end.address_
    %}
    return (squashed_dict_start=squashed_dict_start, squashed_dict_end=squashed_dict_end)
end


func main{range_check_ptr}() -> ():
    %{ initial_dict = {0:1, 1:10, 2:-2}%}
    let (dict_start) = dict_new()
    let dict_end = dict_start
    dict_update{dict_ptr=dict_end}(0, 1, 2)
    dict_update{dict_ptr=dict_end}(0, 2, 3)
    dict_update{dict_ptr=dict_end}(0, 3, 4)
    dict_update{dict_ptr=dict_end}(1, 10, 15)
    dict_update{dict_ptr=dict_end}(1, 15, 20)
    dict_update{dict_ptr=dict_end}(1, 20, 25)
    dict_update{dict_ptr=dict_end}(2, -2, -4)
    dict_update{dict_ptr=dict_end}(2, -4, -8)
    dict_update{dict_ptr=dict_end}(2, -8, -16)
    let (squashed_dict_start, squashed_dict_end) = dict_squash{
        range_check_ptr=range_check_ptr
    }(dict_start, dict_end)
    assert squashed_dict_end[0] = DictAccess(
        key=0, prev_value=1, new_value=4)
    assert squashed_dict_end[1] = DictAccess(
        key=1, prev_value=10, new_value=25)
    assert squashed_dict_end[2] = DictAccess(
        key=2, prev_value=-2, new_value=-16)
    return ()
end
