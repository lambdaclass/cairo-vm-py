%builtins range_check

func assert_250_bit{range_check_ptr}(value):
    const UPPER_BOUND = 2 ** 250
    const SHIFT = 2 ** 128
    const HIGH_BOUND = UPPER_BOUND / SHIFT

    let low = [range_check_ptr]
    let high = [range_check_ptr + 1]

    %{
        # TEST
        from starkware.cairo.common.math_utils import as_int

        # Correctness check.
        value = as_int(ids.value, PRIME) % PRIME
        assert value < ids.UPPER_BOUND, f'{value} is outside of the range [0, 2**250).'

        # Calculation for the assertion.
        ids.high, ids.low = divmod(ids.value, ids.SHIFT)
    %}

    assert [range_check_ptr + 2] = HIGH_BOUND - 1 - high

    # The assert below guarantees that
    #   value = high * SHIFT + low <= (HIGH_BOUND - 1) * SHIFT + 2**128 - 1 =
    #   HIGH_BOUND * SHIFT - SHIFT + SHIFT - 1 = 2**250 - 1.
    assert value = high * SHIFT + low

    let range_check_ptr = range_check_ptr + 3
    return ()
end

func main{range_check_ptr : felt}():
    let x = 5
    let y = 6

    tempvar m = 7
    tempvar n = 7 * 7

    assert_250_bit(250)
    assert_250_bit(132)
    assert_250_bit(x)
    assert_250_bit(y)
    assert_250_bit(3891287381783812783128133211312312318897132873211213278978123)

    return ()
end
