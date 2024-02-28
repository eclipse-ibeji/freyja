# gRPC Cloud Adapter

The gRPC Cloud Adapter is intended to function as a "standard cloud adapter", enabling integration with other services that implement the appropriate APIs. This reduces the need for custom adapter implementations and facilitates integration with non-Rust solutions for other parts of the vehicle system. This library contains an implementation of the `CloudAdapter` trait from the contracts. This adapter also supports service discovery to integrate with service discovery systems such as [Chariott](https://github.com/eclipse-chariott/chariott/blob/main/service_discovery/README.md).

## Contract

This adapter utilizes a gRPC client for the `CloudConnector` service in the [cloud connector v1 protobuf description](../../../interfaces/cloud_connector/v1/cloud_connector.proto). To integrate a cloud connector with this adapter, you will need to implement a gRPC server for this service. Samples can be found in the [Ibeji Example Applications Repository](https://github.com/eclipse-ibeji/ibeji-example-applications/tree/main/cloud_connectors/).

## Configuration

This adapter supports the following configuration settings:

- `service_discovery_id`: The ID of the cloud connector in your service discovery system. The default value is `sdv.cloud_connector/cloud_connector/1.0`.
- `max_retries`: The maximum number of times to retry failed attempts to send data to the server.
- `retry_interval_ms`: The interval between subsequent retry attempts, in milliseconds.

This adapter supports [config overrides](../../../docs/tutorials/config-overrides.md). The override filename is `grpc_cloud_adapter_config.json`, and the default config is located at `res/grpc_cloud_adapter_config.default.json`.
