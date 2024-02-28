# gRPC Mapping Adapter

The gRPC Mapping Adapter is intended to function as a "standard mapping adapter", enabling integration with other services that implement the appropriate APIs. This reduces the need for custom adapter implementations and facilitates integration with non-Rust solutions for other parts of the vehicle system. This library contains an implementation of the `MappingAdapter` trait from the contracts. This adapter also supports service discovery to integrate with service discovery systems such as [Chariott](https://github.com/eclipse-chariott/chariott/blob/main/service_discovery/README.md).

## Contract

This adapter utilizes a gRPC client for the `MappingService` in the [mapping service v1 protobuf description](../../../interfaces/mapping_service/v1/mapping_service.proto). To integrate a mapping service with this adapter, you will need to implement a gRPC server for this service.

## Configuration

This adapter supports the following configuration settings:

- `service_discovery_id`: The ID of the mapping service in your service discovery system. The default value is `sdv.freyja/mapping_service/1.0`.
- `max_retries`: The maximum number of retry attempts when sending data to the server.
- `retry_interval_ms`: The interval between subsequent retry attempts, in milliseconds

This adapter supports [config overrides](../../../docs/tutorials/config-overrides.md). The override filename is `grpc_mapping_adapter_config.json`, and the default config is located at `res/grpc_mapping_adapter_config.default.json`.
