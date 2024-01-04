# Mock Mapping Service

The Mock Mapping Service mocks the behavior of a mapping service as a separate application. This enables a more high-fidelity demo and greater control over the mapping data.

## Configuration

The mock's default config is located at  `res/mock_mapping_config.default.json` and will be copied to the build output automatically. The schema for this config is identical to that of the [In-Memory Mock Mapping Client](../../mapping_clients/in_memory_mock_mapping_client/README.md), and the override mechanisms are the same. Note that the config file name is the same, so using an override at `$FREYJA_HOME/config/mock_mapping_config.json` will apply to both this mock and the in-memory mock.

## Behavior

The behavior of the Mock Mapping Service is largely equivalent to that of the In-Memory Mock Mapping Client linked above, but the count is managed differently depending on whether the application is in interactive mode or not.

### Interactive Mode

In interactive mode, the application maintains an internal count, and only mappings satisfying the condition `begin <= count [< end]` will be returned in the `/mapping` API. Unlike the in-memory client, the internal count is not updated based on how often certain APIs are called but rather by user interaction with the terminal. To increment the application's internal count and potentially change the set of enabled mappings, press <kbd>Enter</kbd> in the application's terminal window. This allows manual control over when the mappings are turned on or off and permits straightforward mocking of more complex scenarios. As a result of this behavior, it is recommended to write configs such that a state change happens each time <kbd>Enter</kbd> is pressed. For example, if a mock scenario has `n` different desired states, then all numbers in the range `0..n-1` should appear as values for at least one `begin` or `end` property. Otherwise pressing <kbd>Enter</kbd> will sometimes have no effect.

**Do not use interactive mode if running this service in a container!** This feature is not compatible with containers and will cause unexpected behavior, including very high resource consumption.

### Non-Interactive Mode

In non-interactive mode, the `begin` and `end` properties in the config are ignored, and all configured mappings are always exposed in the mock's APIs. Furthermore, this means that the application will indicate that there is work to consume when it starts up, but once it's been consumed there will not be any additional work.

This is the default behavior of this application.
