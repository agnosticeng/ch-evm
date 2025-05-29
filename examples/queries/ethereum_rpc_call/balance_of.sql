select 
    ethereum_rpc_call(
        '0xdac17f958d2ee523a2206206994597c13d831ec7', 
        'function balanceOf(address)(uint256)',
        toJSONString(['0x267be1c1d684f78cb4f6a176c4911b741e4ffdc0']),
        -1::Int64,
        'https://eth.llamarpc.com#fail-on-null=true&fail-on-error=true'
)::JSON
settings output_format_arrow_string_as_string=0