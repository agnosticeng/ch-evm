select 
    * 
from iceberg('https://data.agnostic.dev/agnostic-ethereum-mainnet/logs', settings iceberg_use_version_hint=true) 
where block_number between 21000000 and 21099999 
into outfile './tmp/ethereum_mainnet_logs_sample_21000000_21099999.bin' format Native