# Version Update Tool

The version update tool is intended to help with doing batched version updates to all Cargo files in a given repository.

## Usage

This application supports the following arguments:

- `--version`: The new version to apply.
- `--dir` (optional): The directory to search. Defaults to `./`.
- `--dry-run` (optional): When used, the application will show what changes would be made but will not apply them.

Example usage:

```shell
cargo run -p version-update -- \
    --version=1.2.3 \
    --dir=/home/user/freyja \
    --dry-run
```

Example output:

```shell
$ cargo run -p version-update -- --version=0.2.0 --dry-run
    Finished dev [unoptimized + debuginfo] target(s) in 0.20s
     Running `target/debug/version-update --version=0.2.0 --dry-run`
Checking "Cargo.toml"...
        No package version to update

Checking "adapters/cloud/grpc_cloud_adapter/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "adapters/cloud/in_memory_mock_cloud_adapter/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "adapters/data/in_memory_mock_data_adapter/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "adapters/data/managed_subscribe_data_adapter/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "adapters/data/mqtt_data_adapter/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "adapters/data/sample_grpc_data_adapter/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "adapters/digital_twin/grpc_digital_twin_adapter/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "adapters/digital_twin/in_memory_mock_digital_twin_adapter/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "adapters/mapping/grpc_mapping_adapter/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "adapters/mapping/in_memory_mock_mapping_adapter/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "adapters/service_discovery/file_service_discovery_adapter/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "adapters/service_discovery/grpc_service_discovery_adapter/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "build_common/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "common/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "freyja/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "mocks/mock_cloud_connector/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "mocks/mock_digital_twin/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "mocks/mock_mapping_service/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "proc_macros/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "proto/cloud_connector/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "proto/common/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "proto/core_protobuf_data_access/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "proto/mapping_service/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "proto/samples_protobuf_data_access/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "proto/service_discovery_proto/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "test_common/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0

Checking "tools/version_update/Cargo.toml"...
        Would update version: 0.1.0 -> 0.2.0
```
