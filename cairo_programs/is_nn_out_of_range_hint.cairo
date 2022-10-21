%builtins range_check

func main{range_check_ptr: felt}():
    let a = 123

    %{
        # TEST
        memory[ap] = 0 if 0 <= ((-ids.a - 1) % PRIME) < range_check_builtin.bound else 1
    %}

    return ()
end
