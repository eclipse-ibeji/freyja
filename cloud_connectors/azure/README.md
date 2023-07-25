# Azure Cloud Connector

The role of the Cloud Connector is to take the data emitted by Freyja, and update the data of your cloud digital twin which could be hosted in Azure, AWS, etc.

The [MQTT Connector](./mqtt_connector/README.md) and [Digital Twins Connector](./digital_twins_connector/README.md) are two Azure example implementations of a Cloud Connector.

These two examples integrates Freyja with Azure Digital Twins.

However, Freyja is not tightly coupled with Azure and can synchronize data with any cloud solution, provided an appropriate Cloud Connector and adapter are written.

The [MQTT Connector](./mqtt_connector/README.md) relays the data emitted by Freyja to an [Azure Event Grid](https://learn.microsoft.com/en-us/azure/event-grid/overview) using the [MQTT](https://mqtt.org/) protocol to update an Azure Digital Twin instance.

The [Digital Twins Connector](./digital_twins_connector/README.md) updates an Azure Digital Twin instance directly with the data emitted by Freyja.
