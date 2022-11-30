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
    # Create temporary_array in a temporary segment
    local temporary_array : felt*
    %{
        # TEST
        ids.temporary_array = segments.add_temp_segment()
    %}

    # Insert values into temporary_array
    assert temporary_array[0] = 1
    assert temporary_array[1] = 2

    # Create array
    let (array : felt*) = alloc()

    # Insert values into array
    assert array[0] = 50
    assert array[1] = 51

    # Realocate temporary_array into the array segment
    relocate_segment(src_ptr=temporary_array, dest_ptr=array)

    # Assert that the realocated temporary_array gets their values from the array segment
    assert temporary_array[0] = 50
    assert temporary_array[1] = 51
    assert array[2] = 52
    assert temporary_array[2] = 52

    return ()
end
