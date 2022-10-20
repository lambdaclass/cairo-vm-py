func main():
    let a = 1
    let b = 2

    %{
        # TEST
        memory[ap] = 0 if (ids.a % PRIME) <= (ids.b % PRIME) else 1
    %}
    
    return ()
end

