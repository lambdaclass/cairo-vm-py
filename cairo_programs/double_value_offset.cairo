from starkware.cairo.common.alloc import alloc

func main():
    tempvar a = 5
    let (ptr: felt*) = alloc()

    let b = &ptr[a]

    %{
        print("POINTER: ", ids.ptr)
        print("POINTER WITH OFFSET: ", ids.b)
    %}

    %{
        #print(memory.__dict__)
        #print(ap)
        #print("AP:" [ap + (-1)])
        #print("AP2:" [ap + (-4)])
    %}

    let c = 1
    return()
end
