# gRPC Provider Proxy

The gRPC Provider Proxy interfaces with providers which support gRPC. It acts as a consumer for digital twin providers. This proxy supports the `Get` and `Subscribe` operations as defined for the [Ibeji mixed sample](https://github.com/eclipse-ibeji/ibeji/tree/main/samples/mixed). To use this proxy with other providers, those providers will need to support the same API(s) as the provider in that sample (see [Integrating with this Proxy](#integrating-with-this-proxy) for more information).

## Configuration

This proxy supports the following configuration settings:

- `consumer_address`: The address for the proxy's consumer. The proxy's gRPC server will be hosted on this address.
- `advertised_consumer_address`: (Optional) The advertised address for the proxy's consumer. This is the address that will be reported as the callback address to providers, enabling scenarios where the providers should use a different address from the actual hosting address. If not specified, this proxy will default to using the consumer address.

This adapter supports [config overrides](../../../docs/config-overrides.md). The override filename is `grpc_proxy_config.json`, and the default config is located at `res/grpc_proxy_config.default.json`.

## Integrating with this Proxy

This proxy supports the `Publish` API as [defined by the Ibeji samples](https://github.com/eclipse-ibeji/ibeji/blob/main/samples/interfaces/sample_grpc/v1/digital_twin_consumer.proto). In addition, the `value` property of the `PublishRequest` message that providers publish must conform to one of the following structures in order to properly extract the signal value:

- A raw value as a string. For example, `"42"` or `"\"foo\""`.
<!--alex ignore savage-->
- A serialized JSON object with a property not named `$metadata` containing the signal value as a JSON primitive. If there is more than one property that meets these criteria, the first one will be used. For example:

    ```json
    {
        "AmbientAirTemperature": 42,
        "$metadata": {
            "foo": "bar"
        }
    }
    ```

    In the above example, this proxy would extract the value `"42"`.
