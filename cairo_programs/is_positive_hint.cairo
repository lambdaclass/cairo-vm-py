%builtins range_check

func main{range_check_ptr: felt}():
    let value = 123

    %{
        # TEST
        from starkware.cairo.common.math_utils import is_positive
        ids.is_positive = 1 if is_positive(value=ids.value, prime=PRIME, rc_bound=range_check_builtin.bound) else 0
    %}

    return ()
end
