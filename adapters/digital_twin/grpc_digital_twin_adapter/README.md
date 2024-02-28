# gRPC Digital Twin Adapter

The gRPC Digital Twin Adapter is intended to function as a "standard digital twin adapter". It can be used to integrate with in-vehicle digital twin services such as [Ibeji](https://github.com/eclipse-ibeji/ibeji). This adapter also supports service discovery to integrate with service discovery systems such as [Chariott](https://github.com/eclipse-chariott/chariott/blob/main/service_discovery/README.md).

## Ibeji Integration

This adapter implements the [Ibeji In-Vehicle Digital Twin Service API](https://github.com/eclipse-ibeji/ibeji/blob/main/interfaces/invehicle_digital_twin/v1/invehicle_digital_twin.proto) and therefore supports Ibeji integration. In order to use Ibeji with this adapter, you must ensure that the `service_discovery_id` in the config file identifies the Ibeji digital twin service in your service discovery system.

## Configuration

This adapter supports the following configuration settings:

- `service_discovery_id`: The id of the in-vehicle digital twin service in your service discovery system. The default value is `sdv.ibeji/invehicle_digital_twin/1.0`, which corresponds to Ibeji's service discovery ID.
- `max_retries`: The maximum number of times to retry failed attempts to send data to the server.
- `retry_interval_ms`: The interval between subsequent retry attempts, in milliseconds

This adapter supports [config overrides](../../../docs/tutorials/config-overrides.md). The override filename is `grpc_digital_twin_adapter_config.json`, and the default config is located at `res/grpc_digital_twin_adapter_config.default.json`.
