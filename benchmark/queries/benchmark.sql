create dictionary if not exists evm_abi_decoding (
    selector String,
    fullsigs Array(String)
)
primary key selector
source(file(path './tmp/sourcify_20250519.parquet' format 'Parquet'))
lifetime(0)
layout(hashed())

;;

with 
    decoded_logs as (
        select
            JSONExtract(
                evm_decode_event(
                    topics::Array(FixedString(32)),
                    data::String,
                    dictGet(evm_abi_decoding, 'fullsigs', topics[1]::String)
                ),
                'JSON'
            ) as evt
        from file('./tmp/ethereum_mainnet_logs_sample_21000000_21099999.bin', 'Native')
    )

select 
    formatReadableQuantity(count(*)) as total_logs,
    formatReadableQuantity(countIf(evt.error is null)) as decoded_logs,
    formatReadableQuantity(countIf(evt.error is not null)) as undecoded_logs,
    formatReadableQuantity(countIf(evt.error is null) / count(*)) as decoding_ratio
from decoded_logs
settings output_format_arrow_string_as_string=0