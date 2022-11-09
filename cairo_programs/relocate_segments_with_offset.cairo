from starkware.cairo.common.alloc import alloc

func relocate_segment(src_ptr : felt*, dest_ptr : felt*):
    %{ 
        # TEST
        memory.add_relocation_rule(src_ptr=ids.src_ptr, dest_ptr=ids.dest_ptr) 
    %}

    # Add a verifier side assert that src_ptr and dest_ptr are indeed equal.
    assert src_ptr = dest_ptr
    return ()
end

func main():
    alloc_locals
    # Create temporary_array_no_offset in a temporary segment
    local temporary_array : felt*

    %{
        # TEST
        ids.temporary_array = segments.add_temp_segment()
    %}

    # Insert values into temporary_array_no_offset
    assert temporary_array[0] = 1
    assert temporary_array[1] = 2

    # Create array
    let (array : felt*) = alloc()

    # Insert values into array
    assert array[5] = 5
    assert array[6] = 6


    # Realocate temporary_array into the array pointer + 5
    relocate_segment(src_ptr=temporary_array, dest_ptr=(array + 5))

    # Assert that the relocated temporary_array gets their values from the array segment
    assert temporary_array[0] = 5
    assert temporary_array[1] = 6

    return ()
end
