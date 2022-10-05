func main():
    let a : felt = 1
    let b : felt = 2

    %{
        c = ids.a + ids.b
        print("a: ", ids.a)
        print("b: ", ids.b)
        print("c: ", c)
    %}

    return ()
end
