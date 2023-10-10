# GRPC Provider Proxy

The GRPC Provider Proxy interfaces with providers which support GRPC. It acts as a consumer for digital twin providers. This proxy supports the `Get` and `Subscribe` operations as defined for the [Ibeji mixed sample](https://github.com/eclipse-ibeji/ibeji/tree/main/samples/mixed). To use this proxy with other providers, those providers will need to support the same API(s) as the provider in that sample.

## Configuration

This proxy supports the following configuration settings:

- `consumer_address`: The address for the proxy's consumer

This adapter supports [config overrides](../../../docs/config-overrides.md). The override filename is `grpc_proxy_config.json`, and the default config is located at `res/grpc_proxy_config.default.json`.
