func main():
    let a : felt = 1

    %{  
        memory[ap] = 0 if 0 <= (ids.a % PRIME) < range_check_builtin.bound else 1
    %}

    return ()
end
