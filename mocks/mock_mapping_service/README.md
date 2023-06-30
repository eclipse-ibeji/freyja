# Mock Mapping Service

The Mock Mapping Service mocks the behavior of the mapping service in an external application. This enables a more high-fidelity demo and greater control over the mapping data.

## Configuration

The mock can be configured via the `res/config.json` file which is copied to the build output automatically. This file is a list of objects with the following properties:

- `value`: a mapping that should be emitted at some point during the application's lifetime
- `begin`: an integer indicating when to enable the above mapping
- `end`: an optional integer indicating when to disable the above mapping. Set to `null` if you never want the value to "turn off"

Note that this config is identical to the config for the [In-Memory Mock Mapping Client](../../mapping_clients/in_memory_mock_mapping_client/README.md).

## Behavior

The application maintains an internal count, and only mappings satisfying the condition `begin <= count [< end]` will be returned in the `/mapping` API. To increment this count and potentially change the set of enabled mappings, press enter in the application's console. This will also affect the `/work` API, which returns true if the set of mappings has been updated since the last time it was called. This allows manual control over when the mappings are turned on or off and permits straightforward mocking of more complex scenarios. As a result of this behavior, it is recommended to write configs such that a state change happens each time enter is pressed. For example, if a mock scenario has `n` different desired states, then all numbers in the range `0..n-1` should appear as values for at least one `begin` or `end` property.
