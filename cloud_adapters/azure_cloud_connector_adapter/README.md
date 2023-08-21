# Azure Cloud Connector Adapter

This is an example implementation of an adapter for the #[Azure Cloud Connectors](../../cloud_connectors/azure/README.md).

This adapter is used to communicate with an Azure Cloud Connector to synchronize in-vehicle signals to the cloud.

## Prerequisites

### Azure Cloud Connector

You will need to either have #[Azure Digital Twins Connector](../../cloud_connectors/azure/digital_twins_connector/README.md) or #[Azure MQTT Connector](../../cloud_connectors/azure/mqtt_connector/README.md) running.

## Build

1. Before building, please ensure that the `cloud_connector_url` field in `res/azure_cloud_connector_adapter_config.sample.json` matches with the url that the Azure Cloud Connector uses.

```shell
cargo build
```
