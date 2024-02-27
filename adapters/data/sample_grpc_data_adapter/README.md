# Sample gRPC Data Adapter

The Sample gRPC Data Adapter interfaces with providers which support gRPC. It acts as a consumer for digital twin providers. This adapter supports the `Get` and `Subscribe` operations as defined for the [Ibeji mixed sample](https://github.com/eclipse-ibeji/ibeji/tree/main/samples/mixed), which are also used in the [Mock Digital Twin](../../../mocks/mock_digital_twin/README.md). To use this adapter with other providers, those providers will need to support the same API(s) as the provider in that sample (see [Integrating with this Adapter](#integrating-with-this-adapter) for more information).

## Configuration

This adapter supports the following configuration settings:

- `consumer_address`: The address for the adapter's consumer. The adapter's gRPC server will be hosted on this address.
- `advertised_consumer_address`: (Optional) The advertised address for the adapter's consumer. This is the address that will be reported as the callback address to providers, enabling scenarios where the providers should use a different address from the actual hosting address. If not specified, this adapter will default to using the consumer address.

This adapter supports [config overrides](../../../docs/tutorials/config-overrides.md). The override filename is `grpc_data_adapter_config.json`, and the default config is located at `res/grpc_data_adapter_config.default.json`.

## Integrating with this Adapter

This adapter supports the `Publish` API as [defined by the Ibeji samples](https://github.com/eclipse-ibeji/ibeji/blob/main/samples/interfaces/sample_grpc/v1/digital_twin_consumer.proto). In addition, the `value` property of the `PublishRequest` message that providers publish must conform to one of the following structures in order to properly extract the signal value:

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
