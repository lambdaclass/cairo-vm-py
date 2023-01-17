func assert_not_zero(value) {
    %{
        # TEST
        from starkware.cairo.common.math_utils import assert_integer
        assert_integer(ids.value)
        assert ids.value % PRIME != 0, f'assert_not_zero failed: {ids.value} = 0.'
    %}
    if (value == 0) {
        // If value == 0, add an unsatisfiable requirement.
        value = 1;
    }

    return ();
}

func main() {
    assert_not_zero(1);
    assert_not_zero(-1);
    let x = 500 * 5;
    assert_not_zero(x);
    tempvar y = -80;
    assert_not_zero(y);

    return ();
}
