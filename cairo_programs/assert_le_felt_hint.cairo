func main():
    let a = 500
    let b = 600

    %{
        #TEST
        from starkware.cairo.common.math_utils import assert_integer
        assert_integer(ids.a)
        assert_integer(ids.b)
        assert (ids.a % PRIME) < (ids.b % PRIME), f'a = {ids.a % PRIME} is not less than b = {ids.b % PRIME}.'
    %}

    return ()
end
