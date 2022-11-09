
%builtins range_check
from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.math import assert_lt, assert_nn

# Verifies that value appears at least multiplicity times in input.
func verify_multiplicity{range_check_ptr}(
    multiplicity : felt, input_len : felt, input : felt*, value : felt
):
    if multiplicity == 0:
        %{ 
            #TEST
            assert len(positions) == 0 
        %}
        assert_nn(input_len)
        return ()
    end

    alloc_locals
    # Skip to the next appearance.
    local next_item_index
    %{
        #TEST
        current_pos = positions.pop()
        ids.next_item_index = current_pos - last_pos
        last_pos = current_pos + 1
    %}
    assert_nn(next_item_index)
    assert input[next_item_index] = value
    return verify_multiplicity(
        multiplicity=multiplicity - 1,
        input_len=input_len - next_item_index - 1,
        input=&input[next_item_index + 1],
        value=value,
    )
end

func verify_usort{range_check_ptr, output : felt*}(
    input_len : felt, input : felt*, total_visited : felt, multiplicities : felt*, prev : felt
):
    alloc_locals

    if total_visited == input_len:
        return ()
    end

    local value = [output]
    let output = &output[1]
    assert_lt(prev, value)

    local multiplicity = [multiplicities]
    assert_nn(multiplicity - 1)

    %{
        #TEST
        last_pos = 0
        positions = positions_dict[ids.value][::-1]
    %}
    verify_multiplicity(multiplicity=multiplicity, input_len=input_len, input=input, value=value)

    return verify_usort(
        input_len=input_len,
        input=input,
        total_visited=total_visited + multiplicity,
        multiplicities=&multiplicities[1],
        prev=value,
    )
end

func usort{range_check_ptr}(input_len : felt, input : felt*) -> (
    output_len : felt, output : felt*, multiplicities : felt*
):
    alloc_locals
    local output_len
    local output : felt*
    local multiplicities : felt*
    %{ 
        #TEST
        vm_enter_scope(dict(__usort_max_size = globals().get('__usort_max_size'))) 
    %}
    %{
        #TEST
        from collections import defaultdict

        input_ptr = ids.input
        input_len = int(ids.input_len)
        if __usort_max_size is not None:
            assert input_len <= __usort_max_size, (
                f"usort() can only be used with input_len<={__usort_max_size}. "
                f"Got: input_len={input_len}."
            )

        positions_dict = defaultdict(list)
        for i in range(input_len):
            val = memory[input_ptr + i]
            positions_dict[val].append(i)

        output = sorted(positions_dict.keys())
        ids.output_len = len(output)
        ids.output = segments.gen_arg(output)
        ids.multiplicities = segments.gen_arg([len(positions_dict[k]) for k in output])
    %}

    let output_start = output
    verify_usort{output=output}(
        input_len=input_len, input=input, total_visited=0, multiplicities=multiplicities, prev=-1
    )

    %{
        #TEST 
        vm_exit_scope() 
    %}
    return (output_len=output - output_start, output=output_start, multiplicities=multiplicities)
end

func main{range_check_ptr}() -> ():
    alloc_locals
    let (input_array: felt*) = alloc()
    assert input_array[0] = 2
    assert input_array[1] = 1
    assert input_array[2] = 0
    assert input_array[3] = 4
    assert input_array[4] = 3

    let (output_len, output, multiplicities) = usort(input_len=3, input=input_array)

    assert output_len = 3
    assert output[0] = 0
    assert output[1] = 1
    assert output[2] = 2
    assert output[3] = 3
    assert output[4] = 4
    assert multiplicities[0] = 1
    assert multiplicities[1] = 1
    assert multiplicities[2] = 1
    assert multiplicities[3] = 1
    assert multiplicities[4] = 1
    return()
end
