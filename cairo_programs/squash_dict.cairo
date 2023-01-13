%builtins range_check
from starkware.cairo.common.dict_access import DictAccess
from starkware.cairo.common.math import assert_lt_felt
from starkware.cairo.common.alloc import alloc

func squash_dict{range_check_ptr}(
    dict_accesses: DictAccess*, dict_accesses_end: DictAccess*, squashed_dict: DictAccess*
) -> (squashed_dict: DictAccess*) {
    let ptr_diff = [ap];
    %{
        #TEST
        vm_enter_scope()
    %}
    ptr_diff = dict_accesses_end - dict_accesses, ap++;

    if (ptr_diff == 0) {
        // Access array is empty, nothing to check.
        %{
            #TEST 
            vm_exit_scope()
        %}
        return (squashed_dict=squashed_dict);
    }
    let first_key = [fp + 1];
    let big_keys = [fp + 2];
    ap += 2;
    tempvar n_accesses = ptr_diff / DictAccess.SIZE;
    %{
        #TEST
        dict_access_size = ids.DictAccess.SIZE
        address = ids.dict_accesses.address_
        assert ids.ptr_diff % dict_access_size == 0, \
            'Accesses array size must be divisible by DictAccess.SIZE'
        n_accesses = ids.n_accesses
        if '__squash_dict_max_size' in globals():
            assert n_accesses <= __squash_dict_max_size, \
                f'squash_dict() can only be used with n_accesses<={__squash_dict_max_size}. ' \
                f'Got: n_accesses={n_accesses}.'
        # A map from key to the list of indices accessing it.
        access_indices = {}
        for i in range(n_accesses):
            key = memory[address + dict_access_size * i]
            access_indices.setdefault(key, []).append(i)
        # Descending list of keys.
        keys = sorted(access_indices.keys(), reverse=True)
        # Are the keys used bigger than range_check bound.
        ids.big_keys = 1 if keys[0] >= range_check_builtin.bound else 0
        ids.first_key = key = keys.pop()
    %}

    // Call inner.
    if (big_keys != 0) {
        tempvar range_check_ptr = range_check_ptr;
    } else {
        assert first_key = [range_check_ptr];
        tempvar range_check_ptr = range_check_ptr + 1;
    }
    let (range_check_ptr, squashed_dict) = squash_dict_inner(
        range_check_ptr=range_check_ptr,
        dict_accesses=dict_accesses,
        dict_accesses_end_minus1=dict_accesses_end - 1,
        key=first_key,
        remaining_accesses=n_accesses,
        squashed_dict=squashed_dict,
        big_keys=big_keys,
    );
    %{ vm_exit_scope() %}
    return (squashed_dict=squashed_dict);
}

func squash_dict_inner(
    range_check_ptr,
    dict_accesses: DictAccess*,
    dict_accesses_end_minus1: felt*,
    key,
    remaining_accesses,
    squashed_dict: DictAccess*,
    big_keys,
) -> (range_check_ptr: felt, squashed_dict: DictAccess*) {
    alloc_locals;

    let dict_diff: DictAccess* = squashed_dict;

    // Loop to verify chronological accesses to the key.
    // These values are not needed from previous iteration.
    struct LoopTemps {
        index_delta_minus1: felt,
        index_delta: felt,
        ptr_delta: felt,
        should_continue: felt,
    }
    // These values are needed from previous iteration.
    struct LoopLocals {
        value: felt,
        access_ptr: DictAccess*,
        range_check_ptr: felt,
    }

    // Prepare first iteration.
    %{
        #TEST
        current_access_indices = sorted(access_indices[key])[::-1]
        current_access_index = current_access_indices.pop()
        memory[ids.range_check_ptr] = current_access_index
    %}
    // Check that first access_index >= 0.
    tempvar current_access_index = [range_check_ptr];
    tempvar ptr_delta = current_access_index * DictAccess.SIZE;

    let first_loop_locals = cast(ap, LoopLocals*);
    first_loop_locals.access_ptr = dict_accesses + ptr_delta, ap++;
    let first_access: DictAccess* = first_loop_locals.access_ptr;
    first_loop_locals.value = first_access.new_value, ap++;
    first_loop_locals.range_check_ptr = range_check_ptr + 1, ap++;

    // Verify first key.
    key = first_access.key;

    // Write key and first value to dict_diff.
    key = dict_diff.key;
    // Use a local variable, instead of a tempvar, to avoid increasing ap.
    local first_value = first_access.prev_value;
    assert first_value = dict_diff.prev_value;

    // Skip loop nondeterministically if necessary.
    local should_skip_loop;
    %{
        #TEST
        ids.should_skip_loop = 0 if current_access_indices else 1
    %}
    jmp skip_loop if should_skip_loop != 0;

    loop:
    let prev_loop_locals = cast(ap - LoopLocals.SIZE, LoopLocals*);
    let loop_temps = cast(ap, LoopTemps*);
    let loop_locals = cast(ap + LoopTemps.SIZE, LoopLocals*);

    // Check access_index.
    %{
        #TEST
        new_access_index = current_access_indices.pop()
        ids.loop_temps.index_delta_minus1 = new_access_index - current_access_index - 1
        current_access_index = new_access_index
    %}
    // Check that new access_index > prev access_index.
    loop_temps.index_delta_minus1 = [prev_loop_locals.range_check_ptr], ap++;
    loop_temps.index_delta = loop_temps.index_delta_minus1 + 1, ap++;
    loop_temps.ptr_delta = loop_temps.index_delta * DictAccess.SIZE, ap++;
    loop_locals.access_ptr = prev_loop_locals.access_ptr + loop_temps.ptr_delta, ap++;

    // Check valid transition.
    let access: DictAccess* = loop_locals.access_ptr;
    prev_loop_locals.value = access.prev_value;
    loop_locals.value = access.new_value, ap++;

    // Verify key.
    key = access.key;

    // Next range_check_ptr.
    loop_locals.range_check_ptr = prev_loop_locals.range_check_ptr + 1, ap++;

    %{
        #TEST
        ids.loop_temps.should_continue = 1 if current_access_indices else 0
    %}
    jmp loop if loop_temps.should_continue != 0, ap++;

    skip_loop:
    let last_loop_locals = cast(ap - LoopLocals.SIZE, LoopLocals*);

    // Check if address is out of bounds.
    %{
        #TEST
        assert len(current_access_indices) == 0
    %}
    [ap] = dict_accesses_end_minus1 - cast(last_loop_locals.access_ptr, felt);
    [ap] = [last_loop_locals.range_check_ptr], ap++;
    tempvar n_used_accesses = last_loop_locals.range_check_ptr - range_check_ptr;
    %{ assert ids.n_used_accesses == len(access_indices[key]) %}

    // Write last value to dict_diff.
    last_loop_locals.value = dict_diff.new_value;

    let range_check_ptr = last_loop_locals.range_check_ptr + 1;
    tempvar remaining_accesses = remaining_accesses - n_used_accesses;

    // Exit recursion when done.
    if (remaining_accesses == 0) {
        %{
            #TEST
            assert len(keys) == 0
        %}
        return (range_check_ptr=range_check_ptr, squashed_dict=squashed_dict + DictAccess.SIZE);
    }

    let next_key = [ap];
    ap += 1;
    // Guess next_key and check that next_key > key.
    %{
        #TEST
        assert len(keys) > 0, 'No keys left but remaining_accesses > 0.'
        ids.next_key = key = keys.pop()
    %}

    if (big_keys != 0) {
        assert_lt_felt{range_check_ptr=range_check_ptr}(a=key, b=next_key);
        tempvar dict_accesses = dict_accesses;
        tempvar dict_accesses_end_minus1 = dict_accesses_end_minus1;
        tempvar next_key = next_key;
        tempvar remaining_accesses = remaining_accesses;
    } else {
        assert [range_check_ptr] = next_key - (key + 1);
        tempvar range_check_ptr = range_check_ptr + 1;
        tempvar dict_accesses = dict_accesses;
        tempvar dict_accesses_end_minus1 = dict_accesses_end_minus1;
        tempvar next_key = next_key;
        tempvar remaining_accesses = remaining_accesses;
    }

    return squash_dict_inner(
        range_check_ptr=range_check_ptr,
        dict_accesses=dict_accesses,
        dict_accesses_end_minus1=dict_accesses_end_minus1,
        key=next_key,
        remaining_accesses=remaining_accesses,
        squashed_dict=squashed_dict + DictAccess.SIZE,
        big_keys=big_keys,
    );
}

func main{range_check_ptr: felt}() {
    alloc_locals;
    let (dict_start: DictAccess*) = alloc();
    assert dict_start[0] = DictAccess(key=0, prev_value=100, new_value=100);
    assert dict_start[1] = DictAccess(key=1, prev_value=50, new_value=50);
    assert dict_start[2] = DictAccess(key=0, prev_value=100, new_value=200);
    assert dict_start[3] = DictAccess(key=1, prev_value=50, new_value=100);
    assert dict_start[4] = DictAccess(key=0, prev_value=200, new_value=300);
    assert dict_start[5] = DictAccess(key=1, prev_value=100, new_value=150);

    let dict_end = dict_start + 6 * DictAccess.SIZE;

    let (local squashed_dict_start: DictAccess*) = alloc();
    let (squashed_dict_end) = squash_dict{range_check_ptr=range_check_ptr}(
        dict_start, dict_end, squashed_dict_start
    );

    // Check the values of the squashed_dict
    // should be: {0: (100, 300), 1: (50, 150)}
    assert squashed_dict_start[0] = DictAccess(key=0, prev_value=100, new_value=300);
    assert squashed_dict_start[1] = DictAccess(key=1, prev_value=50, new_value=150);
    return ();
}
