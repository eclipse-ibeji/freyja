# Chariott Service Discovery Adapter

The Chariott Service Discovery Adapter integrates with the [Chariott Service Discovery system](https://github.com/eclipse-chariott/chariott/blob/main/service_discovery/README.md) to perform service discovery.

## Service Discovery ID Format

The Chariott Service Discovery Adapter expects service IDs to be in the following format:

<!-- markdownlint-disable-next-line fenced-code-language -->
```
{namespace}/{name}/{version}
```

These parameters correspond to the Chariott service discovery request parameters of the same name.

## Configuration

This adapter supports the following configuration settings:

- `uri`: The URI for Chariott's Service Discovery system.
- `max_retries`: The maximum number of times to retry failed attempts to communicate with Chariott.
- `retry_interval_ms`: The duration between retries in milliseconds.

### Configuration Overrides

This adapter supports [config overrides](../../../docs/tutorials/config-overrides.md). The override filename is `chariott_service_discovery_adapter_config.json`, and the default config is located at `res/chariott_service_discovery_adapter_config.default.json`.
