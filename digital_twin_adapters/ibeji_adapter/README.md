# Ibeji Adapter

The Ibeji Adapter is used to integrate with [Eclipse-Ibeji's In-vehicle Digital Twin Service](https://github.com/eclipse-ibeji/ibeji).

## Behavior

The adapter shares an `entity_map` map with Freyja's emitter that maps `entity_id`s to its entity info. When the emitter receives new mappings from the cartographer, it will update this shared state and insert `None` values for the corresponding ID. The adapter will detect empty entries for each `entity_id` in our `entity_map` then call `find_by_id` to send a request to [Ibeji's In-vehicle Digital Twin Service](https://github.com/eclipse-ibeji/ibeji), to populate the entity info.

### Ibeji Registered as a Chariott Provider

If Ibeji is registered in [Chariott's Service Discovery system](https://github.com/eclipse-chariott/chariott/tree/main/service_discovery), then please set the `invehicle_digital_twin_service_uri` field to `null` in the `res/config.json` file and set the `chariott_service_discovery_uri` field.

Ibeji Adapter will discover Ibeji's In-Vehicle Digital Twin Service URI through Chariott.

Example of config.json:

```json
{
    "invehicle_digital_twin_service_uri": null,
    "chariott_service_discovery_uri": "http://0.0.0.0:50000"
}
```

If both the `invehicle_digital_twin_service_uri` and `chariott_service_discovery_uri` fields are set, then the Ibeji Adapter will directly contact the In-Vehicle Digital Twin Service URI and Chariott will not be used to discover the URI for the In-Vehicle Digital Twin Service.
