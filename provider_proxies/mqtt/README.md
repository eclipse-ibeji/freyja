# MQTT Provider Proxy

The MQTT Provider Proxy interfaces with providers which support MQTT. It acts as a consumer for digital twin providers. This proxy supports the `Subscribe` operations as defined for the [Ibeji property sample](https://github.com/eclipse-ibeji/ibeji/tree/main/samples/property). To use this proxy with other providers, those providers will need to support the same API(s) as the provider in that sample (see [Integrating with this Proxy](#integrating-with-this-proxy) for more information).

## Configuration

This proxy supports the following configuration settings:

- `keep_alive_interval_s`: The keep alive interval for MQTT communications, in seconds

This adapter supports [config overrides](../../docs/config-overrides.md). The override filename is `mqtt_proxy_config.json`, and the default config is located at `res/mqtt_proxy_config.default.json`.

## Integrating with this Proxy

To integrate this proxy with other providers using MQTT, the message that the providers publish must conform to one of the following structures to in order to properly extract the signal value:

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
