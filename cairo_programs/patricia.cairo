%builtins pedersen range_check
from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.cairo_builtins import HashBuiltin
from starkware.cairo.common.patricia import open_edge, ParticiaGlobals
#from starkware.cairo.common.hash import hash2

func main{pedersen_ptr: HashBuiltin*, range_check_ptr}():
    let pow2: felt* = alloc()
    assert pow2[0] = 2
    let globals: ParticiaGlobals* = alloc()
    assert globals[0] = ParticiaGlobals(pow2, 1)
    #open_edge{hash_ptr=pedersen_ptr}(globals, 0)
    let (edge) = open_edge{hash_ptr=pedersen_ptr, range_check_ptr=range_check_ptr}(globals, 0)
    return()
end

