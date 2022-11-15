# This puzzle was part of The Cairo Games Vol. 1 which have already ended.

# Program hash: 0x02bb25e03624218e0211798da4064586ea37958590167006bff1be82e0d99858.

%builtins output pedersen ecdsa

from starkware.cairo.common.cairo_builtins import HashBuiltin, SignatureBuiltin
from starkware.cairo.common.hash import hash2
from starkware.cairo.common.signature import verify_ecdsa_signature

func main{output_ptr : felt*, pedersen_ptr : HashBuiltin*, ecdsa_ptr : SignatureBuiltin*}():
    alloc_locals

    let your_eth_addr = 874739451078007766457464989774322083649278607533249481151382481072868806602
    let signature_r = 1839793652349538280924927302501143912227271479439798783640887258675143576352
    let signature_s = 1819432147005223164874083361865404672584671743718628757598322238853218813979
    let msg = 0000000000000000000000000000000000000000000000000000000000000002

    let (x) = hash2{hash_ptr=pedersen_ptr}('Vamos Argentina',0)
    %{print("ENTER verify_ecdsa_signature")%}
    verify_ecdsa_signature(
        msg,
        874739451078007766457464989774322083649278607533249481151382481072868806602,
        signature_r,
        signature_s,
    )
    %{print("OUT verify_ecdsa_signature")%}


    assert [output_ptr] = your_eth_addr
    let output_ptr = output_ptr + 1

    return ()
end
