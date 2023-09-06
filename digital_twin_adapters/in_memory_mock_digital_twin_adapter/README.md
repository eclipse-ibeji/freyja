# In Memory Mock Digital Twin Adapter

The In Memory Mock Digital Twin Adapter mocks the behavior of an in-vehicle digital twin service (such as Ibeji) from within the memory of the Freyja application. This enables a minimal example scenario when working with Freyja. This library contains an implementation of the `DigitalTwinAdapter` trait from the contracts.

## Configuration

The adapter's config is located at `res/config.json` and will be copied to the build output automatically. This file is a list of `EntityConfig` objects with the following properties:

- `entity`: a digital twin entity that will be exposed to the `find_by_id` API. Entities contain the following properties:
  - `id`: this is used as the key when calling `find_by_id`.
  - `uri`: the uri that is used to invoke a provider. This is a stand-in for whatever the provider contact info is from Ibeji. This is used as the key when calling `subscribe` and `get` in the [In-Memory Provider Proxy](../../provider_proxies/in_memory_mock_provider_proxy/).
  - `operation`: the operation that should be used to access this entity. Supported values are `Get` and `Subscribe`.
  - `protocol`: the communication protocol that should be used to access this entity. For this particular adapter, the value should always be `in-memory`.
  - `name` and `description`: these are currently unused by Freyja. They are included for completeness and parity with Ibeji's Digital Twin Service contract and may potentially be logged.
