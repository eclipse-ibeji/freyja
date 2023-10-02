# In-Memory Mock Digital Twin Adapter

The In-Memory Mock Digital Twin Adapter mocks the behavior of an in-vehicle digital twin service (such as Ibeji) from within the memory of the Freyja application. This enables a minimal example scenario when working with Freyja. This library contains an implementation of the `DigitalTwinAdapter` trait from the contracts.

## Configuration

This adapter supports the following configuration settings:

- `values`: a list of entities to use. Each entry in the list is an object with the following properties:
  - `entity`: a digital twin entity that will be exposed to the `find_by_id` API. Entities contain the following properties:
    - `id`: this is used as the key when calling `find_by_id`.
    - `uri`: the uri that is used to invoke a provider. This is a stand-in for whatever the provider contact info is from Ibeji. This is used as the key when calling `subscribe` and `get` in the [In-Memory Provider Proxy](../../provider_proxies/in_memory_mock_provider_proxy/).
    - `operation`: the operation that should be used to access this entity.
    - `protocol`: the communication protocol that should be used to access this entity. For this particular adapter, the value should always be `in-memory`.
    - `name` and `description`: these are currently unused by Freyja. They are included for completeness and parity with Ibeji's Digital Twin Service contract and may potentially be logged.

This adapter supports [config overrides](../../docs/config-overrides.md). The override filename is `in_memory_digital_twin_config.json`, and the default config is located at `res/in_memory_digital_twin_config.default.json`.