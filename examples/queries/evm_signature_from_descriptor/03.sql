select evm_signature_from_descriptor(
    JSONExtract(
        evm_descriptor_from_fullsig('function transfer(address,uint256)(bool)'),
        'value',
        'String'
    )
)
settings output_format_arrow_string_as_string=0