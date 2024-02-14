# File Service Discovery Adapter

The File Service Discovery Adapter retrieves service URIs from a config file. This is intended for use in testing scenarios or as a backup service discovery adapter if other service discovery adapters fail to retrieve a URI.

## Configuration

This adapter supports the following configuration settings:

- `services`: a map with string keys and values which maps service ids to their URIs

### Configuration Overrides

This adapter supports [config overrides](../../../docs/tutorials/config-overrides.md). The override filename is `file_service_discovery_adapter_config.json`, and the default config is located at `res/file_service_discovery_adapter_config.default.json`.
