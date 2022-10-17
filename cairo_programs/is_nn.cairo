%builtins range_check

from starkware.cairo.common.math_cmp import is_nn

func main{range_check_ptr: felt}():
    let (d) = is_nn(0)
    assert d = 1
    let (e) = is_nn(88)
    assert e = 1
    let (f) = is_nn(-88)
    assert f = 0

    return ()
end
