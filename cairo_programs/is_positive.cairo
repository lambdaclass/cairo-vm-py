%builtins range_check

func abs_value{range_check_ptr}(value) -> (abs_value : felt):
    tempvar is_positive : felt
    %{
        # TEST
        from starkware.cairo.common.math_utils import is_positive
        ids.is_positive = 1 if is_positive(
            value=ids.value, prime=PRIME, rc_bound=range_check_builtin.bound) else 0
    %}
    if is_positive == 0:
        tempvar new_range_check_ptr = range_check_ptr + 1
        tempvar abs_value = value * (-1)
        [range_check_ptr] = abs_value
        let range_check_ptr = new_range_check_ptr
        return (abs_value=abs_value)
    else:
        [range_check_ptr] = value
        let range_check_ptr = range_check_ptr + 1
        return (abs_value=value)
    end
end

func main{range_check_ptr: felt}():
    let a = abs_value(5)
    let b = abs_value(-5)
    let c = abs_value(123)
    let d = abs_value(-123)

    assert a = b
    assert c = d

    return ()
end
