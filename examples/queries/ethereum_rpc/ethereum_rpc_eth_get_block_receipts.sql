select
    ethereum_rpc(
        'eth_getBlockReceipts', 
        [evm_hex_encode_int(number)], 
        'https://eth.llamarpc.com'
    )::JSON
from numbers(20764111, 5)
settings output_format_arrow_string_as_string=0