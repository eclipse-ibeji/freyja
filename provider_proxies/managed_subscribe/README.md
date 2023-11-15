# Managed Subscribe Provider Proxy

The Managed Subscribe Provider Proxy interfaces with providers which support gRPC and utilize the [Managed Subscribe](https://github.com/eclipse-ibeji/ibeji/tree/main/samples/managed_subscribe) module in Ibeji. It acts as a consumer for digital twin providers. This proxy supports the `Managed Subscribe` operation as defined for the [Ibeji managed subscribe sample](https://github.com/eclipse-ibeji/ibeji/tree/main/samples/managed_subscribe). To use this proxy with other providers, those providers will need to support the same API(s) as the provider in that sample (see [Integrating with this Proxy](#integrating-with-this-proxy) for more information).

## Configuration

This proxy supports the following configuration settings:

- `frequency_constraint_type`: The type of frequency constraint to use. Defaults to `frequency_ms`.
- `frequency_constraint_value`: The frequency at which to get data at. Defaults to `3000` ms.

The above values define the frequency at which the proxy requests a provider to publish at.

This adapter supports [config overrides](../../docs/config-overrides.md). The override filename is `managed_subscribe_proxy_config.json`, and the default config is located at `res/managed_subscribe_proxy_config.default.json`.

## Integrating with this Proxy

To integrate this proxy with other providers using gRPC and Managed Subscribe, the message that the providers publish must conform to one of the following structures to in order to properly extract the signal value:

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
