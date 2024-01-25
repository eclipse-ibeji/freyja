# MQTT Data Adapter

The MQTT Data Adapter interfaces with providers which support MQTT. It acts as a consumer for digital twin providers. This adapter supports the `Subscribe` operations as defined for the [Ibeji property sample](https://github.com/eclipse-ibeji/ibeji/tree/main/samples/property). To use this adapter with other providers, those providers will need to support the same API(s) as the provider in that sample (see [Integrating with this Adapter](#integrating-with-this-adapter) for more information).

## Configuration

This adapter supports the following configuration settings:

- `keep_alive_interval_s`: The keep alive interval for MQTT communications, in seconds

This adapter supports [config overrides](../../../docs/tutorials/config-overrides.md). The override filename is `mqtt_data_adapter_config.json`, and the default config is located at `res/mqtt_data_adapter_config.default.json`.

## Integrating with this Adapter

To integrate this adapter with other providers using MQTT, the message that the providers publish must conform to one of the following structures to in order to properly extract the signal value:

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

    In the above example, this adapter would extract the value `"42"`.
