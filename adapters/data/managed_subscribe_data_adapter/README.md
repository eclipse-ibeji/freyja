# Managed Subscribe Data Adapter

The Managed Subscribe Data Adapter interfaces with providers which support gRPC and utilize the [Managed Subscribe](https://github.com/eclipse-ibeji/ibeji/tree/main/samples/managed_subscribe) module in Ibeji. It acts as a consumer for digital twin providers. This adapter supports the `Managed Subscribe` operation as defined for the [Ibeji managed subscribe sample](https://github.com/eclipse-ibeji/ibeji/tree/main/samples/managed_subscribe). To use this adapter with other providers, those providers will need to support the same API(s) as the provider in that sample (see [Integrating with this Adapter](#integrating-with-this-adapter) for more information).

## Configuration

This adapter supports the following configuration settings:

- `frequency_constraint_type`: The type of frequency constraint to use. Defaults to `frequency_ms`.
- `frequency_constraint_value`: The frequency at which the data is transferred. Defaults to `3000` ms.

The above values define the frequency at which the adapter requests a provider to transfer data.

This adapter supports [config overrides](../../../docs/tutorials/config-overrides.md). The override filename is `managed_subscribe_data_adapter_config.json`, and the default config is located at `res/managed_subscribe_data_adapter_config.default.json`.

## Integrating with this Adapter

To integrate this adapter with other providers using gRPC and Managed Subscribe, the message that the providers publish must conform to one of the following structures to in order to properly extract the signal value:

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
