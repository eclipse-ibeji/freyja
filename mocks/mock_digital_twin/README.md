# Mock Digital Twin

The Mock Digital Twin mocks the behavior of the in-vehicle digital twin services (e.g. Ibeji) in a separate process. This enables functionality similar to the in-memory mock, but with finer control over the behavior of the mocked data.

## Configuration

The mock can be configured via the `res/config.json` file which is copied to the build output automatically. This file is a list of objects with the following properties:

- `value`: a mock digital twin provider that should be enabled at some point during the application's lifecycle. The properties of this object are identical to those for the [In-Memory Mock Digital Twin Adapter](../../digital_twin_adapters/in_memory_mock_digital_twin_adapter/README.md), excluding the `provider_url`.

- `begin`: an integer indicating when to enable the above mock provider
- `end`: an optional integer indicating when to disable the above mock provider. Set to `null` if you never want the provider to "turn off"

## Behavior

The application maintains an internal count, and only mock providers satisfying the condition `begin <= count [< end]` will be enabled for the `/providers` APIs. To increment this count and potentially change the set of enabled providers, press enter in the application's console. This allows manual control over when the mock providers are turned on or off and permits straightforward mocking of more complex scenarios. As a result of this behavior, it is recommended to write configs such that a state change happens each time enter is pressed. For example, if a mock scenario has `n` different desired states, then all numbers in the range `0..n-1` should appear as values for at least one `begin` or `end` property.

In addition, the mock also maintains a count of the number of times each provider has been invoked, and returns a value that is a function of this count. In this way, the behavior of the `generate_signal_values()` API is identical to that of the In-Memory Digital Twin Adapter.

The `/providers` endpoint is to retrieve a provider's DTDL (Digital Twin Definition Language).

Providers that support the `Subscribe` operation can allow clients of `Mock Digital Twin Adapter` to send a request to the `/providers/subscribe/{provider_id}` endpoint, and the server will periodically publish the providers' values to the client for a subscribe.

Similarly, providers that support the `Get` operation can allow clients of `Mock Digital Twin Adapter` to send a request to the `/providers/get/{provider_id}` endpoint. The server will publish the providers' values only once to the client for a get. If the client wishes to retrieve the values again, then the client would need to send another request.
