select
    ethereum_rpc(
        'eth_getBlockByNumber', 
        ['"finalized"', 'false'], 
        'https://eth.llamarpc.com#fail-on-null=true'
    )::JSON
settings output_format_arrow_string_as_string=0