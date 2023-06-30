# Mock Digital Twin Adapter

The Mock Digital Twin Adapter mocks the behavior of a consumer in the in-vehicle digital twin service (e.g. Ibeji). This performs calls to the [Mock Digital Twin](../../mocks/mock_digital_twin/README.md) to find the Digital Twin Definition Language (DTDL) for entities and retrieves data from them based on the supported data access operations. This library contains an implementation of the `DigitalTwinAdapter` trait from the contracts.

## Behavior

The adapter shares a `providers_dtdl` hashmap with the emitter which maps `provider_id`s to `Option` entries. When the emitter receives new mappings from the cartographer, it will update this shared state and insert `None` values for the corresponding ID. The adapter will then detect these `None` values and call `find_by_id` to populate the DTDL.

The client inside the adapter sends a request to the provider's address, which is obtained from the DTDL.

If a provider supports the `Subscribe` operation, then a one-time subscribe is performed.
If a provider supports the `Get` operation, then the http client will periodically send a get request to the external server to get the signal values.
If a provider supports both the `Subscribe` and `Get` operations, only the `Subscribe` operation will be selected.

In either case, the adapter runs a listener that will capture values pushed from the Mock Digital Twin and cache them for the emitter. In both the `Subscribe` and `Get` operations, a callback url is in the request message body which the Mock Digital Twin will use to publish signal values when available.
