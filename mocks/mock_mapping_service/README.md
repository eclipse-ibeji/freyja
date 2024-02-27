# Mock Mapping Service

The Mock Mapping Service mocks the behavior of a mapping service as a separate application. This enables a more high-fidelity demo and greater control over the mapping data compared to the in-memory mock.

The Mock Mapping Service implements the [Mapping Service API](../../interfaces/mapping_service/v1/mapping_service.proto), making it compatible with the [gRPC Mapping Adapter](../../adapters/mapping/grpc_mapping_adapter/README.md).

## Configuration

This mock supports the following configuration:

- `mapping_server_authority`: The authority that will be used for hosting the mapping service.
- `values`: The list of mapping entries. The schema for this property is identical to the schema for the `values` property in the [In-Memory Mock Mapping Adapter](../../adapters/mapping/in_memory_mock_mapping_adapter/README.md)

This mock supports [config overrides](../../docs/tutorials/config-overrides.md). The override filename is `mock_mapping_config.json`, and the default config is located at `res/mock_mapping_config.default.json`. Note that this filename is the same as the one for the In-Memory Mock Mapping Adapter and that the override mechanisms are the same, so the same override files can be used for both adapters (the in-memory adapter will ignore the `mapping_server_authority` value).

## Behavior

The behavior of the Mock Mapping Service is largely equivalent to that of the In-Memory Mock Mapping Adapter linked above, but the count is managed differently depending on whether the application is in interactive mode or not.

### Non-Interactive Mode

Non-interactive mode is the default behavior of this application. In non-interactive mode, the `begin` and `end` properties in the config are ignored, and all configured mappings are always exposed in the mock's APIs. Furthermore, this means that the application will indicate that there is work to consume when it starts up, but once it's been consumed there will never be any additional work.

### Interactive Mode

In interactive mode, the application maintains an internal count, and only mappings satisfying the condition `begin <= count [< end]` will be returned in the `/mapping` API. Unlike the in-memory adapter, the internal count is not updated based on how often certain APIs are called but rather by user interaction with the terminal. To increment the application's internal count and potentially change the set of enabled mappings, press <kbd>Enter</kbd> in the application's terminal window. This allows manual control over when the mappings are turned on or off and permits straightforward mocking of more complex scenarios. As a result of this behavior, it is recommended to write configs such that a state change happens each time <kbd>Enter</kbd> is pressed. For example, if a mock scenario has `n` different desired states, then all numbers in the range `0..n-1` should appear as values for at least one `begin` or `end` property. Otherwise pressing <kbd>Enter</kbd> will sometimes have no effect.

**Do not use interactive mode if running this service in a container!** This feature is not compatible with containers and will cause unexpected behavior, including very high resource consumption.

## Build and Run

To build and run the Mock Mapping Service in non-interactive mode, run the following command:

```shell
cargo run -p mock-mapping-service
```

To enable interactive mode, run the same command with the `--interactive` argument:

```shell
cargo run -p mock-mapping-service -- --interactive
```
