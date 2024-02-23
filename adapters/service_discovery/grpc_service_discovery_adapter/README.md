# gRPC Service Discovery Adapter

The gRPC Service Discovery Adapter is intended to function as a 
"standard service discovery adapter". It can be used to integrate with service discovery systems such as the [Chariott Service Discovery system](https://github.com/eclipse-chariott/chariott/blob/main/service_discovery/README.md).

## Chariott Integration

This adapter utilizes the `Discover` function of the [Chariott Service Registry API](https://github.com/eclipse-chariott/chariott/blob/main/service_discovery/proto/core/v1/service_registry.proto) and therefore supports Chariott integration. In order to use Chariott with this adapter, you must ensure that the `uri` entry in the config matches the URI configured for Chariott's service discovery system.

## Service Discovery ID Format

The gRPC Service Discovery Adapter expects service IDs to be in the following format:

<!-- markdownlint-disable-next-line fenced-code-language -->
```
{namespace}/{name}/{version}
```

These parameters correspond to the `DiscoveryRequest` parameters of the same name.

## Configuration

This adapter supports the following configuration settings:

- `uri`: The URI for Chariott's Service Discovery system.
- `max_retries`: The maximum number of times to retry failed attempts to communicate with Chariott.
- `retry_interval_ms`: The duration between retries in milliseconds.

### Configuration Overrides

This adapter supports [config overrides](../../../docs/tutorials/config-overrides.md). The override filename is `grpc_service_discovery_adapter_config.json`, and the default config is located at `res/grpc_service_discovery_adapter_config.default.json`.
