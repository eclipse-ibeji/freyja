# Mock Digital Twin Adapter

The Mock Digital Twin Adapter mocks the behavior of a consumer in the in-vehicle digital twin service (e.g. Ibeji). This performs calls to the [Mock Digital Twin](../../mocks/mock_digital_twin/README.md) to find entity info for entities. This library contains an implementation of the `DigitalTwinAdapter` trait from the contracts.

## Behavior

The adapter shares an `entity_map` map with Freyja's emitter that maps `entity_id`s to its entity info. When the emitter receives new mappings from the cartographer, it will update this shared state and insert `None` values for the corresponding ID. The adapter will detect empty entries for each `entity_id` in our `entity_map` then call `find_by_id` to send a request to the [HTTP Mock In-Vehicle Digital Twin's](../../mocks/mock_digital_twin/) URI, to populate the entity info.
