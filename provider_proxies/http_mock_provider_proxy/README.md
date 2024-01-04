# HTTP Mock Provider Proxy

The HTTP Mock Provider Proxy mocks the behavior of a proxy which communicates with providers via HTTP. This is intended for use with the [Mock Digital Twin](../../mocks/mock_digital_twin/).

## Configuration

This proxy supports the following configuration settings:

- `proxy_callback_address`: The address for the proxy. This is the address that the Mock Digital Twin will use for callbacks. This should be a URI with no scheme and no port.
- `starting_port`: The starting port number to use when creating proxies. The factory will increment the port it uses each time a proxy is created.

This adapter supports [config overrides](../../docs/config-overrides.md). The override filename is `http_mock_proxy_config.json`, and the default config is located at `res/http_mock_proxy_config.default.json`.
