# Azure Cloud Connector

The role of the Cloud Connector is to take the data emitted by Freyja, and update the data of your cloud digital twin which could be hosted in Azure, AWS, etc.

The [MQTT Connector](./mqtt_connector/README.md) and [Digital Twins Connector](./digital_twins_connector/README.md) are two sample implementations using Azure Digital Twins.

These two examples integrates Freyja with Azure Digital Twins.

However, Freyja is not tightly coupled with Azure and can synchronize data with any cloud solution, provided an appropriate Cloud Connector and adapter are written.

The [MQTT Connector](./mqtt_connector/README.md) relays the data emitted by Freyja to an [Azure Event Grid](https://learn.microsoft.com/en-us/azure/event-grid/overview) using the [MQTT](https://mqtt.org/) protocol. Data coming into the Event Grid will trigger an Azure function which updates the Azure Digital Twin instance.

The [Digital Twins Connector](./digital_twins_connector/README.md) updates an Azure Digital Twin instance directly with the data emitted by Freyja.

## Prerequisites for Automated Deployment of Azure Resources

The deployment scripts in the `{freyja-root-dir}/cloud_connectors/azure/scripts` directory will automate the deployment of necessary Azure resources depending on which Azure Cloud Connector sample you wish to use.

To run the deployment scripts, you will need to complete each prerequisite section specified below.

Alternatively, you can create Azure resources via the Azure portal. Please see [Manual Azure Digital Twins Setup](./digital_twins_connector/README.md#manual-azure-digital-twins-setup) for the Digital Twins Connector sample, and [Manual Deployment of Azure Key Vault, Event Grid, and Azure Function App](./mqtt_connector/README.md#manual-deployment-of-azure-key-vault-event-grid-and-azure-function-app) for the MQTT Connector sample.

### Azure CLI and Extensions

You must install the following:

* [Azure CLI](https://learn.microsoft.com/en-us/cli/azure/install-azure-cli)

* [Azure IoT CLI Extension](https://github.com/Azure/azure-iot-cli-extension)

### Azure Resource Group Role-Based Access Control

You will need to be an Owner or a Contributor for your Azure resource group to deploy Azure resources using the scripts. Please see [Azure built-in roles](https://learn.microsoft.com/en-us/azure/role-based-access-control/built-in-roles) for more details.

## Automated Deployment of Azure Resources

Please see [Automated Azure Digital Twins Setup](./digital_twins_connector/README.md#automated-azure-digital-twins-setup) for the Digital Twins Connector sample, and [Automated Deployment of Azure Key Vault, Event Grid, and Azure Function App](./mqtt_connector/README.md#automated-deployment-of-azure-key-vault-event-grid-and-azure-function-app) for the MQTT Connector sample.

If you experience permission or deployment errors, try running the script again as sometimes it takes a while for some dependencies to be fully deployed. If you use the same name or identifier for each Azure resource, the script will not create additional copies of that Azure resource.

You may also follow the [Manual Azure Digital Twins Setup](./digital_twins_connector/README.md#manual-azure-digital-twins-setup) for the Digital Twins Connector sample, or the [Manual Deployment of Azure Key Vault, Event Grid, and Azure Function App](./mqtt_connector/README.md#manual-deployment-of-azure-key-vault-event-grid-and-azure-function-app) for the MQTT Connector sample sections to deploy the respective Azure resource that is failing to be deployed by the script.
