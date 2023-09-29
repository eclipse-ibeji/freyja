# In-Memory Mock Mapping Client

The In-Memory Mock Mapping Client mocks the behavior of a mapping service from within the memory of the Freyja application. This enables a minimal example scenario when working with Freyja. This library contains an implementation of the `MappingClient` trait from the contracts.

## Configuration

This adapter supports the following configuration settings:

- `values`: a list of mappings to use. Each entry in the list is an object with the following properties:
  - `begin`: an integer indicating when to enable the `value`
  - `end`: an optional integer indicating when to disable the `value`. Set to `null` if you never want the value to "turn off"
  - `value`: a mapping that should be emitted at some point during the application's lifetime. This has the following properties:
    - `source`: the ID of the entity that will be used as the source for this mapping. This should match something that's retrievable with the `find_by_id` API of the digital twin adapter that you're using.
    - `target`: a set of key-value pairs that will be passed to the cloud adapter. This is completely free-form, and will potentially be used by the cloud adapter to help with addressing the correct digital twin instance and/or properties for upstream data emissions.
    - `interval_ms`: the interval at which the entity should be queried for changes
    - `emit_on_change`: a value indicating whether data emission should be skipped if the value hasn't changed since the last emission. Set to true to enable this behavior.
    - `conversion`: a conversion that should be applied. Set to `null` if no conversion is needed. Otherwise the conversion is configured with the `mul` and `offset` properties, and the value `y` that is emitted is calculated as `y = mul * x + offset`. Note that conversions are only supported for signal values which can be parsed as `f64`.

This adapter supports [config overrides](../../../docs/config-overrides.md). The override filename is `mock_mapping_config.json`, and the default config is located at `res/mock_mapping_config.default.json`.

## Behavior

The client maintains an internal count, and only mappings satisfying the condition `begin <= count [< end]` will be returned in the `get_mapping` API. This count is incremented every time the `check_for_work` API is invoked. This will also affect the `check_for_work` API, which returns true if the set of mappings has changed since the last time it was called. This effectively means that the state can potentially change with each loop of the cartographer.
