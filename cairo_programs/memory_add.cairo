func alloc() -> (ptr : felt*):
    %{ 
        # TEST
        memory[ap] = segments.add() 
    %}
    ap += 1
    return (ptr=cast([ap - 1], felt*))
end

func main():
    let a : felt* = alloc()
    return ()
end
