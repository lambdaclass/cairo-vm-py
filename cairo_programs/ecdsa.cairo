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

    verify_ecdsa_signature(
        msg,
        your_eth_addr,
        signature_r,
        signature_s,
    )


    assert [output_ptr] = your_eth_addr
    let output_ptr = output_ptr + 1

    return ()
end
