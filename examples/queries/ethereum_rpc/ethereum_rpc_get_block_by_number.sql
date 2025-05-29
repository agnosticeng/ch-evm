select
    number,
    ethereum_rpc(
        'eth_getBlockByNumber', 
        [evm_hex_encode_int(number), 'false'], 
        'https://eth.llamarpc.com'
    )::JSON as res
from numbers(20000000, 10)
settings output_format_arrow_string_as_string=0