# Mock Digital Twin Adapter

The Mock Digital Twin Adapter acts as a client for the [Mock Digital Twin](../../mocks/mock_digital_twin/README.md) when getting entity info with the `find_by_id` API. This library contains an implementation of the `DigitalTwinAdapter` trait from the contracts.

## Config

The adapter's config is located at `res/config.json` and will be copied to the build output automatically. This file contains the following properties:

- `base_uri_for_digital_twin_server`: the base uri for the Mock Digital Twin Service