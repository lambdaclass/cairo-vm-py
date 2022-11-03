%builtins range_check

func assert_le{range_check_ptr}(a, b):
    %{
        # TEST
        a = ids.a % PRIME
        b = ids.b % PRIME
        assert a <= b, f'a = {a} is not less than or equal to b = {b}.'
    %}
    return ()
end

func main{range_check_ptr}():
    let a: felt = 1
    let b: felt = 2
    assert_le(a, b)
    return ()
end
