from starkware.cairo.common.alloc import alloc

func main() {
    tempvar a = 5;
    let (ptr: felt*) = alloc();

    let b = &ptr[a];

    %{
        print("POINTER: ", ids.ptr)
        print("POINTER WITH OFFSET: ", ids.b)
    %}

    return ();
}
