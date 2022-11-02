from starkware.cairo.common.alloc import alloc

func unsafe_keccak(data : felt*, length : felt) -> (low : felt, high : felt):
    alloc_locals
    local low
    local high
    %{
        from eth_hash.auto import keccak

        data, length = ids.data, ids.length

        if '__keccak_max_size' in globals():
            assert length <= __keccak_max_size, \
                f'unsafe_keccak() can only be used with length<={__keccak_max_size}. ' \
                f'Got: length={length}.'

        keccak_input = bytearray()
        for word_i, byte_i in enumerate(range(0, length, 16)):
            word = memory[data + word_i]
            n_bytes = min(16, length - byte_i)
            assert 0 <= word < 2 ** (8 * n_bytes)
            keccak_input += word.to_bytes(n_bytes, 'big')

        hashed = keccak(keccak_input)
        ids.high = int.from_bytes(hashed[:16], 'big')
        ids.low = int.from_bytes(hashed[16:32], 'big')
    %}
    return (low=low, high=high)
end

func main():
    alloc_locals

    let (data : felt*) = alloc()

    assert data[0] = 500 
    assert data[1] = 2
    assert data[2] = 3
    assert data[3] = 6
    assert data[4] = 1
    assert data[5] = 4444

    let (low : felt, high : felt) = unsafe_keccak(data, 6)

    assert low = 182565855334575837944615807286777833262
    assert high = 90044356407795786957420814893241941221

    return ()
end
    

