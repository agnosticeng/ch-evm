## What is this benchmark

## Download datasets

```sh
wget -o ./tmp/ethereum_mainnet_logs_sample_21000000_21099999.bin https://pub-00ad5fed1d0944e7b87fb47b09d9e246.r2.dev/ch-evm-benchmark/ethereum_mainnet_logs_sample_21000000_21099999.bin
wget -o ./tmp/sourcify_20250519.parquet https://pub-00ad5fed1d0944e7b87fb47b09d9e246.r2.dev/ch-evm-benchmark/sourcify_20250519.parquet
```

## Build ch-evm

```sh
make bundle RELEASE=true
```

## Run benchmark query with clickhouse-local

```sh
export BUNDLE_PATH="./tmp/bundle" # change it to any other bundle (eg: clickhouse-evm)

clickhouse local \
    --log-level=debug \
    --path="$BUNDLE_PATH/var/lib/clickhouse" \
    --queries-file="./benchmark/queries/benchmark.sql" \
    --format=Markdown \
    -- \
    --user_defined_executable_functions_config="$BUNDLE_PATH/etc/clickhouse-server/*_function.*ml" \
    --user_scripts_path="$BUNDLE_PATH/var/lib/clickhouse/user_scripts"
```