# Ibeji Adapter

The Ibeji Adapter is used to integrate with [Eclipse-Ibeji's In-vehicle Digital Twin Service](https://github.com/eclipse-ibeji/ibeji).

## Behavior

The adapter shares an `entity_map` map with Freyja's emitter that maps `entity_id`s to its entity info. When the emitter receives new mappings from the cartographer, it will update this shared state and insert `None` values for the corresponding ID. The adapter will detect empty entries for each `entity_id` in our `entity_map` then call `find_by_id` to send a request to [Ibeji's In-vehicle Digital Twin Service](https://github.com/eclipse-ibeji/ibeji), to populate the entity info.

### Ibeji Without Chariott

The Ibeji Adapter will use Ibeji's In-Vehicle Digital Digital URI if the `service_type` field in `res/config.json` is set to `"InVehicleDigitalTwinService`

Example of config.json:

```json
{
    "service_type": "InVehicleDigitalTwinService",
    "uri": "http://0.0.0.0:5010",
    "max_retries": 5,
    "duration_between_retries_ms": 1000
}
```

### Ibeji With Chariott

If Ibeji is registered in [Chariott's Service Discovery system](https://github.com/eclipse-chariott/chariott/blob/main/service_discovery/README.md)and you wish to discover Ibeji through Chariott, then please set the `service type` field to `ChariottDiscoveryService` in the `res/config.json` file, value for the `uri` field, and the metadata for Ibeji.

The Ibeji Adapter will discover Ibeji's In-Vehicle Digital Twin Service URI through Chariott.

Example of config.json:

```json
{
    "service_type": "ChariottDiscoveryService",
    "uri": "http://0.0.0.0:50000",
    "max_retries": 5,
    "duration_between_retries_ms": 1000,
    "metadata": {
        "namespace": "sdv.ibeji",
        "name": "digital_twin",
        "version": "1.0"
    }
}
```
