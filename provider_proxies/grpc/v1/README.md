# gRPC Provider Proxy

The gRPC Provider Proxy interfaces with providers which support gRPC. It acts as a consumer for digital twin providers. This proxy supports the `Get` and `Subscribe` operations as defined for the [Ibeji mixed sample](https://github.com/eclipse-ibeji/ibeji/tree/main/samples/mixed). To use this proxy with other providers, those providers will need to support the same API(s) as the provider in that sample.

## Configuration

This proxy supports the following configuration settings:

- `consumer_address`: The address for the proxy's consumer. The proxy's gRPC server will be hosted on this address.
- `advertised_consumer_address`: (Optional) The advertised address for the proxy's consumer. This is the address that will be reported as the callback address to providers, enabling scenarios where the providers should use a different address from the actual hosting address. If not specified, this proxy will default to using the consumer address.

This adapter supports [config overrides](../../../docs/config-overrides.md). The override filename is `grpc_proxy_config.json`, and the default config is located at `res/grpc_proxy_config.default.json`.
