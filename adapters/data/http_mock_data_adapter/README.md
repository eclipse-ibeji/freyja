# HTTP Mock Data Adapter

The HTTP Mock Data Adapter mocks the behavior of an adapter which communicates with providers via HTTP. This is intended for use with the [Mock Digital Twin](../../../mocks/mock_digital_twin/).

## Configuration

This adapter supports the following configuration settings:

- `callback_address`: The address for the adapter. This is the address that the Mock Digital Twin will use for callbacks. This should be a URI with no scheme and no port.
- `starting_port`: The starting port number to use when creating adapters. The factory will increment the port it uses each time an adapter is created.

This adapter supports [config overrides](../../../docs/tutorials/config-overrides.md). The override filename is `http_mock_data_adapter_config.json`, and the default config is located at `res/http_mock_data_adapter_config.default.json`.
