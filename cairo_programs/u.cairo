
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
):
    alloc_locals
    local output_len
    local output : felt*
    local multiplicities : felt*
    %{
        #TEST
        from collections import defaultdict
        input_ptr = ids.input
        input_len = int(ids.input_len)
        positions_dict = defaultdict(list)
        positions_dict[1].append(2)
        print(positions_dict)
        for i in range(input_len):
            val = memory[input_ptr + i]
            positions_dict[val].append(i)
        print("Everything is Okay")
        output = sorted(positions_dict.keys())
        print("Everything is Okay 2")
        ids.output_len = len(output)
        print("Everything is Okay 3")
        ids.output = segments.gen_arg(output)
        print("Everything is Okay 4")
        for k in output:
            print(len(positions_dict[k]))
        print("Everything is Okay 4.2")
        #print(len(positions_dict[0]))
        #print(len(positions_dict[1]))
        #print(len(positions_dict[2]))
        lista = list(positions_dict)
        #positions_dict = [[1],[1,2],[0]]
        #print(positions_dict)
        positions_dict = [1,2,3]
        print("Everything is Okay 4.3")
        lista = [positions_dict[k] for k in range(2)]
        #lista = [positions_dict for k in output]
        print(lista)
        print("Everything is Okay 4.5")
        #ids.multiplicities = segments.gen_arg([len(positions_dict[k]) for k in output])
        print("Everything is Okay 5")
    %}
    return ()
end

func main{range_check_ptr}() -> ():
    alloc_locals
    let (input_array: felt*) = alloc()
    assert input_array[0] = 2
    assert input_array[1] = 1
    assert input_array[2] = 0
    assert input_array[3] = 4
    assert input_array[4] = 3

    usort(input_len=3, input=input_array)
    return()
end
