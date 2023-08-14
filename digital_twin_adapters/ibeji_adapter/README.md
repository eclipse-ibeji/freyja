# Ibeji Adapter

The Ibeji Adapter is used to integrate with [Eclipse-Ibeji's In-vehicle Digital Twin Service](https://github.com/eclipse-ibeji/ibeji).

## Behavior

The adapter shares an `entity_map` map with Freyja's emitter that maps `entity_id`s to its entity info. When the emitter receives new mappings from the cartographer, it will update this shared state and insert `None` values for the corresponding ID. The adapter will detect empty entries for each `entity_id` in our `entity_map` then call `find_by_id` to send a request to [Ibeji's In-vehicle Digital Twin Service](https://github.com/eclipse-ibeji/ibeji), to populate the entity info.

### Ibeji Without Chariott

By default, running `cargo build` will copy the `ibeji_adapter_config.sample.json` file from the `res` directory and paste it into the `{freyja-root-dir}/target/debug` directory. Please ensure that the `uri` field in the `ibeji_adapter_config.json` file in `{freyja-root-dir}/target/debug` matches with the URI that Ibeji's In-Vehicle Digital Twin service uses.

The Ibeji Adapter will use Ibeji's In-Vehicle Digital Digital URI if the `service_type` field in `res/config.json` is set to `"InVehicleDigitalTwinService`

### Ibeji With Chariott

If Ibeji is registered in [Chariott's Service Discovery system](https://github.com/eclipse-chariott/chariott/blob/main/service_discovery/README.md) and you wish to discover Ibeji through Chariott, then copy the `ibeji_adapter_config_with_chariott.json` file from the `res` directory, and paste it into the `{freyja-root-dir}/target/debug` directory.

Please ensure that the `uri` field in the `ibeji_adapter_config.json` file in `{freyja-root-dir}/target/debug` matches with the URI that Chariott's Serice Discovery uses.

The Ibeji Adapter will discover Ibeji's In-Vehicle Digital Twin Service URI through Chariott.
