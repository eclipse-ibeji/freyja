# Mock Cloud Connector

The Mock Cloud Connector mocks the behavior of a Cloud Connector. This enables functionality similar to the [In-Memory Mock Cloud Adapter](../../adapters/cloud/in_memory_mock_cloud_adapter/README.md) while also utilizing the standard Freyja application.

The Mock Cloud Connector implements the [Cloud Connector API](../../interfaces/cloud_connector/v1/cloud_connector.proto), making it compatible with the [gRPC Cloud Adapter](../../adapters/cloud/grpc_cloud_adapter/README.md).

## Configuration

This mock supports the following configuration:

- `server_authority`: The authority that will be used for hosting the mock cloud connector service.

This mock supports [config overrides](../../docs/tutorials/config-overrides.md). The override filename is `mock_cloud_connector_config.json`, and the default config is located at `res/mock_cloud_connector_config.default.json`.

## Behavior

This cloud connector prints the requests that it receives to the console, enabling users to verify that data is flowing from Freyja to the cloud connector. As a mock, this cloud connector does not have any cloud connectivity.

## Build and Run

To build and run the Mock Cloud Connector, run the following command:

```shell
cargo run -p mock-cloud-connector
```
