# Mock Mapping Service Client

Together with the matching service, the Mock Mapping Service client mocks the behavior of the mapping service separate from the Freyja application. This enables a higher-fidelity demo with finer control over the behavior of the mocked components. This library contains an implementation of the `MappingClient` trait from the contracts.

For more information about the mock service, see [the README for the Mock Mapping Service](../../mocks/mock_mapping_service/README.md)

## Prerequsites

The HTTP client library used in this implementation requires Open-SSL 1.0.1, 1.0.2, 1.1.0, or 1.1.1 with headers. This requires the following additional setup to build the client on Ubuntu:

```shell
sudo apt-get install -y pkg-config libssl-dev
```

For instructions on other operating systems, see the full documentation [here](https://docs.rs/openssl/latest/openssl/#automatic)
