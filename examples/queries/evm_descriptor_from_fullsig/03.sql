select evm_descriptor_from_fullsig(
    JSONExtract(
        evm_signature_from_descriptor('{"inputs":[{"internalType":"address","name":"arg0","type":"address"},{"internalType":"uint256","name":"arg1","type":"uint256"}],"name":"transfer","outputs":[{"internalType":"bool","name":"arg0","type":"bool"}],"type":"function"}'),
        'value',
        'fullsig',
        'String'
    )
)
settings output_format_arrow_string_as_string=0