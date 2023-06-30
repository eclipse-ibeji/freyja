# Ibeji Adapter

The Ibeji Adapter is used to integrate with [Eclipse-Ibeji's In-vehicle Digital Twin Service](https://github.com/eclipse-ibeji/ibeji).

## Behavior

The adapter shares an `entity_map` map with Freyja's emitter that maps `entity_id`s to its entity info. When the emitter receives new mappings from the cartographer, it will update this shared state and insert `None` values for the corresponding ID. The adapter will detect empty entries for each `entity_id` in our `entity_map` then call `find_by_id` to send a request to [Ibeji's In-vehicle Digital Twin Service](https://github.com/eclipse-ibeji/ibeji), to populate the entity info.
