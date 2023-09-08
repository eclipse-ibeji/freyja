# Mock Mapping Service

The Mock Mapping Service mocks the behavior of a mapping service as a separate application. This enables a more high-fidelity demo and greater control over the mapping data.

## Configuration

The mock can be configured via the `res/config.json` file which is copied to the build output automatically. The schema for this config is identical to that of the [In-Memory Mock Mapping Client](../../mapping_clients/in_memory_mock_mapping_client/README.md).

## Behavior

The behavior of the Mock Mapping Service is largely identical to that of the In-Memory Mock Mapping Client linked above. The one notable exception is that the internal count is not updated based on how often certain APIs are called but rather by user interaction with the terminal. To increment the application's internal count and potentially change the set of enabled mappings, press <kbd>Enter</kbd> in the application's terminal window.

The application maintains an internal count, and only mappings satisfying the condition `begin <= count [< end]` will be returned in the `/mapping` API. To increment this count and potentially change the set of enabled mappings, press enter in the application's console. This allows manual control over when the mappings are turned on or off and permits straightforward mocking of more complex scenarios. As a result of this behavior, it is recommended to write configs such that a state change happens each time enter is pressed. For example, if a mock scenario has `n` different desired states, then all numbers in the range `0..n-1` should appear as values for at least one `begin` or `end` property. Otherwise pressing <kbd>Enter</kbd> will sometimes have no effect.
