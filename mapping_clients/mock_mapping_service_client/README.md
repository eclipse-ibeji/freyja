# Mock Mapping Service Client

The Mock Mapping Service Client acts as a client for the [Mock Mapping Service](../../mocks/mock_mapping_service/README.md). This enables a higher-fidelity demo with finer control over the behavior of the mocked components. This library contains an implementation of the `MappingClient` trait from the contracts.

## Prerequisites

The HTTP client library used in this implementation requires Open-SSL 1.0.1, 1.0.2, 1.1.0, or 1.1.1 with headers. On Ubuntu, this requires the following additional setup:

```shell
sudo apt-get install -y pkg-config libssl-dev
```

For instructions on other operating systems, see the full documentation [here](https://docs.rs/openssl/latest/openssl/#automatic)

## Config

The adapter's default config is located at `res/mock_mapping_client_config.default.json` and will be copied to the build output automatically. This file contains the following properties:

- `max_retires`: the maximum number of retries permitted when attempting to call the mock service
- `retry_interval_ms`: the interval between subsequent retry attempts, in milliseconds
- `mock_mapping_service_url`: the url for the Mock Mapping Service Client

You can override the default values by defining your own `mock_mapping_client_config.json`. The adapter will probe for and unify config in this order, with values near the end of the list taking higher precedence:

- The default config
- A `mock_mapping_client_config.json` file in the working directory of the executable (for example, the directory you were in when you ran the `cargo run` command)
- `$FREYJA_HOME/config/mock_mapping_client_config.json`. If you have not set a `$FREYJA_HOME` directory, this defaults to:
  - Unix: `$HOME/.freyja/config/mock_mapping_client_config.json`
  - Windows: `%USERPROFILE%\.freyja\config\mock_mapping_client_config.json` (note that Windows support is not guaranteed by Freyja or this adapter)
