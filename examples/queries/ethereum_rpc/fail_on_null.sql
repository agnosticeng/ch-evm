select
    ethereum_rpc(
        method, 
        [evm_hex_encode_int(block_number)], 
        'https://eth.llamarpc.com#fail-on-null=true'
    )::JSON
from values(
    'method String, block_number UInt64', 
    ('eth_getBlockReceipts', 20000000),
    ('eth_getBlockReceipts', 20000000000)
)
settings output_format_arrow_string_as_string=0