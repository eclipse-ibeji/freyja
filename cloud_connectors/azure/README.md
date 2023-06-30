# Azure Cloud Connector

The role of the Cloud Connector is to take the data emitted by Freyja, and update the data of your cloud digital twin which could be hosted in Azure, AWS, etc.

The [Digital Twins Connector](./digital_twins_connector/README.md) is an Azure example implementations of a Cloud Connector.

These two examples integrates Freyja with Azure Digital Twins.

However, Freyja is not tightly coupled with Azure and can synchronize data with any cloud solution, provided an appropriate Cloud Connector and adapter are written.

The [Digital Twins Connector](./digital_twins_connector/README.md) updates an Azure Digital Twin instance directly with the data emitted by Freyja.
