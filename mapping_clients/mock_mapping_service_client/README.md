# Mock Mapping Service Client

The Mock Mapping Service Client acts as a client for the [Mock Mapping Service](../../mocks/mock_mapping_service/README.md). This enables a higher-fidelity demo with finer control over the behavior of the mocked components. This library contains an implementation of the `MappingClient` trait from the contracts.

## Prerequisites

The HTTP client library used in this implementation requires Open-SSL 1.0.1, 1.0.2, 1.1.0, or 1.1.1 with headers. On Ubuntu, this requires the following additional setup:

```shell
sudo apt-get install -y pkg-config libssl-dev
```

For instructions on other operating systems, see the full documentation [here](https://docs.rs/openssl/latest/openssl/#automatic)

## Config

This adapter supports the following configuration settings:

- `max_retires`: the maximum number of retries permitted when attempting to call the mock service
- `retry_interval_ms`: the interval between subsequent retry attempts, in milliseconds
- `mock_mapping_service_url`: the url for the Mock Mapping Service Client

This adapter supports [config overrides](../../docs/config-overrides.md). The override filename is `mock_mapping_client_config.json`, and the default config is located at `res/mock_mapping_client_config.default.json`.
