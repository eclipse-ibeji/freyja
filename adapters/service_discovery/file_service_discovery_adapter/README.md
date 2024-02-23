# File Service Discovery Adapter

The File Service Discovery Adapter retrieves service URIs from a config file. This is intended for use in testing scenarios or as a backup service discovery adapter if other service discovery adapters fail to retrieve a URI.

## Service Discovery ID Format

There is no required format for service IDs for this adapter. The adapter will use the requested IDs to look up the service from the config. If using this adapter as a fallback for other service discovery adapters, it's recommended to use config keys which exactly match the IDs expected by the primary adapter(s). The default config for this adapter matches with the format expected by the [gRPC Service Discovery Adapter](../grpc_service_discovery_adapter/README.md).

## Configuration

This adapter supports the following configuration settings:

- `services`: a map with string keys and values which maps service ids to their URIs

### Configuration Overrides

This adapter supports [config overrides](../../../docs/tutorials/config-overrides.md). The override filename is `file_service_discovery_adapter_config.json`, and the default config is located at `res/file_service_discovery_adapter_config.default.json`.
