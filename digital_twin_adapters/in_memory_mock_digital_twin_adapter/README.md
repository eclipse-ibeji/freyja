# In-Memory Mock Digital Twin Adapter

The In-Memory Mock Digital Twin Adapter mocks the behavior of an in-vehicle digital twin service (such as Ibeji) from within the memory of the Freyja application. This enables a minimal example scenario when working with Freyja. This library contains an implementation of the `DigitalTwinAdapter` trait from the contracts.

## Configuration

This adapter supports the following configuration settings:

- `values`: A list of entities to use. Each entry in the list is an object with the following properties:
  - `entity`: A digital twin entity that will be exposed to the `find_by_id` API. Entities contain the following properties:
    - `id`: This is used as the key when calling `find_by_id`.
    - `name` and `description`: These are currently unused by Freyja. They are included for completeness and parity with Ibeji's Digital Twin Service contract and may potentially be logged.
    - `endpoints`: A list of endpoints that this entity supports. Each entry in the list is an object with the following properties:
      - `protocol`: The communication protocol that should be used to access this entity. For most use cases with this adapter, the value of this property will be `in-memory`.
      - `operations`: A list of operations that can be used to access this entity.
      - `uri`: The uri that is used to invoke a provider. This is used as the key when calling functions on the proxies. If you're using the `in-memory` protocol, requests are not actually submitted to this uri so it does not need to be a real endpoint.

This adapter supports [config overrides](../../docs/config-overrides.md). The override filename is `in_memory_digital_twin_config.json`, and the default config is located at `res/in_memory_digital_twin_config.default.json`.
