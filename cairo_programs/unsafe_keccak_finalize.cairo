from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.keccak import KeccakState
from starkware.cairo.common.uint256 import Uint256

func unsafe_keccak_finalize(keccak_state : KeccakState) -> (res : Uint256):
    alloc_locals
    local low
    local high
    %{
        from eth_hash.auto import keccak
        keccak_input = bytearray()
        n_elms = ids.keccak_state.end_ptr - ids.keccak_state.start_ptr
        for word in memory.get_range(ids.keccak_state.start_ptr, n_elms):
            keccak_input += word.to_bytes(16, 'big')
        hashed = keccak(keccak_input)
        ids.high = int.from_bytes(hashed[:16], 'big')
        ids.low = int.from_bytes(hashed[16:32], 'big')
    %}
    return (res=Uint256(low=low, high=high))
end

func main():
    alloc_locals

    let (data : felt*) = alloc()

    assert data[0] = 0 
    assert data[1] = 1
    assert data[2] = 2

    let keccak_state = KeccakState(start_ptr=data, end_ptr=data + 2) 

    let res : Uint256 = unsafe_keccak_finalize(keccak_state)

    assert res.low = 17219183504112405672555532996650339574
    assert res.high = 235346966651632113557018504892503714354

    return ()
end
