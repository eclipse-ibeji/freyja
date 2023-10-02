# Mock Digital Twin Adapter

The Mock Digital Twin Adapter acts as a client for the [Mock Digital Twin](../../mocks/mock_digital_twin/README.md) when getting entity info with the `find_by_id` API. This library contains an implementation of the `DigitalTwinAdapter` trait from the contracts.

## Config

This adapter supports the following configuration settings:

- `digital_twin_service_uri`: the base uri for the Mock Digital Twin Service

This adapter supports [config overrides](../../docs/config-overrides.md). The override filename is `mock_digital_twin_adapter_config.json`, and the default config is located at `res/mock_digital_twin_adapter_config.default.json`.
