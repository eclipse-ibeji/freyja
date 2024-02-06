# Ibeji Adapter

The Ibeji Adapter is used to integrate with the [Ibeji In-Vehicle Digital Twin Service](https://github.com/eclipse-ibeji/ibeji), and optionally [Chariott](https://github.com/eclipse-chariott/chariott) to discover Ibeji.

## Configuration

This adapter supports two different configuration schemas depending on how you want to discover the In-Vehicle Digital Twin Service:

### Without Chariott

To bypass Chariott and use a configuration value to specify the In-Vehicle Digital Twin Service URI, you must specify the following configuration:

- `service_discovery_method`: Set this value to `"Config"`.
- `uri`: The URI for the In-Vehicle Digital Twin Service.
- `max_retries`: The maximum number of times to retry failed attempts to communicate with the In-Vehicle Digital Twin Service.
- `retry_interval_ms`: The duration between retries in milliseconds.

### Using Chariott

To use Chariott to discover the In-Vehicle Digital Twin Service, you must specify the following configuration:

- `service_discovery_method`: Set this value to `"ChariottServiceDiscovery"` to use Chariott.
- `uri`: The URI for Chariott's Service Discovery system.
- `max_retries`: The maximum number of times to retry failed attempts to communicate with Chariott or the In-Vehicle Digital Twin Service.
- `retry_interval_ms`: The duration between retries in milliseconds.
- `metadata`: Metadata for the discovery operation:
  - `namespace`: The namespace for the In-Vehicle Digital Twin Service.
  - `name`: The service name for the In-Vehicle Digital Twin Service.
  - `version`: The version of the In-Vehicle Digital Twin Service to query for.

An example of a configuration file that uses Chariott can be found at `res/ibeji_adapter_config.chariott_sample.json`.

### Configuration Overrides

This adapter supports the same [config override method](../../../docs/tutorials/config-overrides.md) as the Freyja mocks. The override filename is `ibeji_adapter_config.json`, and the default config is located at `res/ibeji_adapter_config.default.json`.
