func assert_not_equal(a, b):
    %{
        # TEST
        from starkware.cairo.lang.vm.relocatable import RelocatableValue
        both_ints = isinstance(ids.a, int) and isinstance(ids.b, int)
        both_relocatable = (
            isinstance(ids.a, RelocatableValue) and isinstance(ids.b, RelocatableValue) and
            ids.a.segment_index == ids.b.segment_index)
        assert both_ints or both_relocatable, \
            f'assert_not_equal failed: non-comparable values: {ids.a}, {ids.b}.'
        assert (ids.a - ids.b) % PRIME != 0, f'assert_not_equal failed: {ids.a} = {ids.b}.'
    %}
    if a == b:
        # If a == b, add an unsatisfiable requirement.
        a = a + 1
    end

    return ()
end

func main():
    assert_not_equal(1, 6)
    assert_not_equal(2, 5)
    let x = 500 * 5
    assert_not_equal(x, 9)
    tempvar y = -80
    assert_not_equal(y, 10)

    return ()
end
