# In-Memory Mock Provider Proxy

The In-Memory Mock Provider Proxy mocks a provider within the memory of a Freyja application. This is intended for use with the [In-Memory Mock Digital Twin Adapter](../../adapters/digital_twin/in_memory_mock_digital_twin_adapter/).

## Configuration

This proxy supports the following configuration settings:

- `signal_update_frequency_ms`: The frequency between updates to signal values in milliseconds. This mimics the publish frequency of a real provider.
- `entities`: A list of entity configuration items with the following properties:
  - `entity_id`: The id of an entity to mock
  - `values`: Defines the values that the entity's signal should take. This can take one of two forms:
    - `Static`: The signal's value is a constant, configured as a string
    - `Stepwise`: The signal's value will change, increasing and decreasing cyclically by a set value between upper and lower bounds. When using this setting the following additional configuration is required:
      - `start`: The starting value of the signal. This can be either the upper or lower bound.
      - `end`: The other bound for the signal value
      - `delta`: The amount to add to the signal value at each iteration. If this operation would exceed the specified bounds, then the signal value saturates at the boundary value.

This adapter supports [config overrides](../../docs/config-overrides.md). The override filename is `in_memory_mock_proxy_config.json`, and the default config is located at `res/in_memory_mock_proxy_config.default.json`.

## Behavior

The application maintains an internal count for each entity to generate its signal values. An entity's signal value is derived from its count based on the entity's `values` configuration.

Entities that support the `Subscribe` operation will mock a subscribe operation. The proxy will periodically update signal values and the associated internal counter at the frequency specified by `signal_update_frequency_ms`.

Entities that support the `Get` operation will provide their values on-demand. The internal count for these entities is updated each time the value is requested, and is not based on a set frequency like the `Subscribe` entities.
