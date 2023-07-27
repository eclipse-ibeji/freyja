# Azure MQTT Cloud Connector

This is an example implementation of an Azure Cloud Connector.

Freyja is not tightly coupled with Azure and can synchronize data with any cloud solution, provided an appropriate Cloud Connector and adapter are written.

For this Cloud Connector sample, you will use Azure to deploy an Azure Key Vault, an Event Grid with MQTT v5, and a Function App.

## Architecture

When signals are propagated from Freyja, the Azure MQTT Cloud Connector will publish these signals to an Azure Event Grid topic using the MQTT protocol. When signals are published to a topic on the Event Grid, an Azure Function gets triggered and updates an Azure Digital Twin instance with the data emitted by Freyja.

Below is a high-level diagram that illustrates Freyja communicating with the Azure MQTT Cloud Connector:

![Component Diagram](../../../docs/diagrams/azure_mqtt_cloud_connector.svg)

## Prerequisites

### Azure Digital Twins Deployed

In your Azure Digital Twins resource, you will also need to create digital twin instances. Sample DTDL models are located in the `{freyja-root-dir}/cloud_connectors/azure/sample-dtdl` directory.

Please see [Automated Azure Digital Twins Setup](../digital_twins_connector/README.md#automated-azure-digital-twins-setup) or [Manual Azure Digital Twins Setup](../digital_twins_connector/README.md#manual-azure-digital-twins-setup) for additional info on setting up Azure Digital Twins.

### Self-Signed X.509 Certificate

Please see steps 1-3 in [Azure Event Grid with MQTT](#2-azure-event-grid-with-mqtt) for additional info on generating an X.509 self-signed certificate and getting its thumbprint.

## Automated Deployment of Azure Key Vault, Event Grid, and Azure Function App

Before starting this section, please view [Prerequisites for Automated Deployment of Azure Resources](../README.md#prerequisites-for-automated-deployment-of-azure-resources).

1. Sign in with Azure CLI. Follow the prompts after entering the following command.

```shell
az login --use-device-code
```

1. You can either pass in a config or arguments to the `mqtt_connector_setup.sh` script.

If you wish to pass a config, then please copy the `mqtt_connector_setup.template.json` in the `{freyja-root-dir}/cloud_connectors/azure/scripts` directory, and fill in the placeholders.

```shell
cd {freyja-root-dir}/cloud_connectors/azure/scripts
chmod +x mqtt_connector_setup.sh
./mqtt_connector_setup.sh -c <MQTT_CONFIG_JSON_PATH>
```

Field descriptions:

* `resource_group`: The name of your resource group.

* `subscription_id`: The subscription ID that your resource group is under.

* `digital_twins_name`: The name of your Digital Twins resource.

* `thumbprint_of_cert_in_der_format`: The thumbprint of your X.509 certificate in DER format.

* `storage_account_name`: The desired name for the Storage Account you want to create.

* `function_app_name`: The desired name for the Azure Function App you want to create.

* `key_vault_name`: The desired name for the Key Vault you want to create.

* `event_grid_topic`: The desired name for the Event Grid Topic you want to create.

* `event_grid_subscription_name`: The desired name for the Event Grid Subscription you want to create.

* `event_grid_namespace`: The desired name for the Event Grid Namespace you want to create.

* `mqtt_client_authentication_name`: The desired name for the Event Grid Client Authentication you want to use to authenticate.

If you wish to pass in arguments, then please do the following:

```shell
cd {freyja-root-dir}/cloud_connectors/azure/scripts
chmod +x mqtt_connector_setup.sh
./mqtt_connector_setup.sh -r myResourceGroup -s mySubscriptionId -dt-n myDigitalTwinsName \
     -t myThumbprint -st-a-n myStorageAccountName -f-a-n myFunctionAppName \
     -k-v-n myKeyVaultName -e-g-t myEventGridTopic -e-g-s-n myEventGridSubscriptionName \
     -e-g-n myEventgridNamespace -m-c-a-n myMqttClientAuthenticationName
```

## Manual Deployment of Azure Key Vault, Event Grid, and Azure Function App

### 1. Azure Key Vault

1. Follow the *Open instance in Azure Digital Twins Explorer* section under [Set up Azure Digital Twins](https://learn.microsoft.com/en-us/azure/digital-twins/quickstart-azure-digital-twins-explorer#set-up-azure-digital-twins) to get the Azure Digital Twin URL of your Azure Digital Twin instance.

1. [Create an Azure Key Vault using the Azure portal](https://learn.microsoft.com/en-us/azure/key-vault/general/quick-create-portal).

1. Create a secret with `ADT-INSTANCE-URL` as the name, and the value should be the **Azure Digital Twin URL** that you obtained in step 1.

You have successfully deployed your Key Vault if you see an `ADT-ISTANCE-URL` secret and the status of that secret is enabled.

### 2. Azure Event Grid with MQTT

1. Create a private key. Replace the `{PrivateKeyName}` placeholder with the name you wish to use.

    ```shell
    openssl genpkey -out {PrivateKeyName}.key -algorithm RSA -pkeyopt rsa_keygen_bits:2048
    ```

1. Create a certificate signing request. Replace the placeholders with their respective values, and fill in the prompts of the certificate signing request.

    ```shell
    openssl req -new -key {PrivateKeyName}.key -out {CertificateSigningRequestName}.csr
    ```

1. Create an X.509 self-signed certificate. Replace the placeholders with their respective values.

    ```shell
    openssl x509 -req -days 365 -in {CertificateSigningRequestName}.csr -signkey {PrivateKeyName}.key -out {CertificateName}.cer
    ```

1. Get the thumbprint of your certificate in DER format. You will need the thumbprint when [creating a client](https://learn.microsoft.com/en-us/azure/event-grid/mqtt-publish-and-subscribe-portal#create-clients) for your Event Grid in step 6.

    ```shell
    openssl x509 -in {CertificateName}.cer -outform DER -out {CertificateName}.crt
    sha256sum {CertificateName}.crt | awk '{print $1}'
    rm {CertificateName}.crt
    ```

1. Follow the [Quickstart: Publish and subscribe to MQTT messages on Event Grid Namespace with Azure portal](https://learn.microsoft.com/en-us/azure/event-grid/mqtt-publish-and-subscribe-portal) guide for creating an Azure Event Grid, topic namespace, and client. You can skip the *Generate sample client certificate and thumbprint* section as you have generated a self-signed certificate in steps 1-3.

1. Once you have successfully deployed an Event Grid namespace, navigate to it then copy the `MQTT Hostname` field. You will need it later for the `mqtt_event_grid_host_name` field in the configuration file that is described in the [Configuration](#configuration) section.

1. In the [Create clients](https://learn.microsoft.com/en-us/azure/event-grid/mqtt-publish-and-subscribe-portal#create-clients) section, use the thumbprint you obtained in step 4 for thumbprint match authentication. Also keep note of what you set for the **Client Authentication Name**. You will need it later for the `mqtt_client_authentication_name` field in the configuration file that is described in the [Configuration](#configuration) section.

1. When you [create a topic space](https://learn.microsoft.com/en-us/azure/event-grid/mqtt-publish-and-subscribe-portal#create-topic-spaces), keep note of the name you used for the **topic template**. You will need it later for the `mqtt_event_grid_topic` field in the configuration file that is described in the [Configuration](#configuration) section.

You have successfully deployed your Event Grid Namespace if you have a publisher permission binding, a client and a client group, and a topic space.
Navigate to the client that you have created in your Event Grid Namespace, and validate that the `Client Certificate Authentication Validation Scheme` is set to `Thumbprint Match`, and the thumbprint matches to your self-signed certificate obtained in [Azure Event Grid with MQTT](#2-azure-event-grid-with-mqtt).

### 3. Azure Function App

1. [Create an Azure Function app](https://learn.microsoft.com/en-us/azure/event-grid/custom-event-to-function#create-azure-function-app) that triggers your Azure Event Grid. Ensure you set the Runtime stack to .NET and version 6.0.

1. Replace the code in your Azure Function `run.cs` with the code in the `res/azure_function/run.cs` folder of this repo.

1. Add the following files `res/azure_function/function.json` and `res/azure_function/function.csproj` to your Azure Function.

1. Go back to your Azure Function App homepage, and click on **Configuration** on the side-panel.

1. Click on **New application setting**.

1. Set the name to `KEYVAULT_SETTINGS`, and the value to `@Microsoft.KeyVault(SecretUri={YOUR_ADT_INSTANCE_URI_SECRET_IN_KEY_VAULT})`

1. Replace the placeholder `{YOUR_ADT_INSTANCE_URI_SECRET_IN_KEY_VAULT}` with the secret URI to your `ADT-INSTANCE-URL` secret in Key Vault obtained from step 3 of [Azure Key Vault](#1-azure-key-vault). To find the URI to your `ADT-INSTANCE-URL` secret, click on your Key Vault then Secrets. Click on ADT-INSTANCE-URL -> Current version, and copy the secret identifier.

You have successfully deployed your Azure Function App if you see the files in steps 1-2 uploaded. If you navigate to `Configuration` under the `Settings` of your Azure Function App then under `Application settings`, you see a green check mark beside the `Key vault Reference` label for `KEYVAULT_SETTINGS`.

### 4. Enable Managed System Identity in Azure Function App

Your Azure Function App will need the Azure Digital Twins Data Owner role to read/write to your Azure Digital Twin instances.
Also your Function App will need the Key Vault Reader role to read the `ADT-INSTANCE-URL` secret you had set up in step 3 of [Azure Key Vault](#1-azure-key-vault).

1. Navigate to the homepage of your Azure Function App.

1. Click on **Identity** on the side-panel, then click on **Azure role assignments**.

1. Click **On** button for the Status to enable Managed System Identity.

1. Click on **Add role assignment**.

1. Use the following settings for the Azure Digital Twins Data Owner role:
    * Scope: Resource Group
    * Subscription: {YOUR_SUBSCRIPTION}
    * Resource group: {YOUR_RESOURCE_GROUP}
    * Role: Azure Digital Twins Data Owner

1. Repeat step 4, but use the following settings for the Key Vault Reader role:
    * Scope: Key Vault
    * Subscription: {YOUR_SUBSCRIPTION}
    * Resource: {YOUR_KEYVAULT}
    * Role: Key Vault Reader

## Build

```shell
cargo build
```

## Configuration

Whether you followed the [Automated Deployment of Azure Key Vault, Event Grid, and Azure Function App](#automated-deployment-of-azure-key-vault-event-grid-and-azure-function-app), or the [Manual Deployment of Azure Key Vault, Event Grid, and Azure Function App](#manual-deployment-of-azure-key-vault-event-grid-and-azure-function-app), you will still need to follow the configuration steps below.

1. Change directory to the directory with the build artifacts `{freyja-root-dir}/target/debug`. Replace `{freyja-root-dir}` with the repository's root directory.

    ```shell
    cd {freyja-root-dir}/target/debug
    ```

1. After building the MQTT Connector, you should see a `mqtt_config.json` file in your `{freyja-root-dir}/target/debug`. If you do not see the `mqtt_config.json` file in `{freyja-root-dir}/target/debug`, you can create one manually by copying the `res/mqtt_config.template.json` file and pasting it into `{freyja-root-dir}/target/debug`.

1. Replace the placeholders in your `mqtt_config.json` with their respective values.

    Field descriptions:
    <!--alex ignore host-hostess-->
    * `grpc_server_authority`: The gRPC server authority you wish to use to host the MQTT Connector's gRPC server. Example `"grpc_server_authority": "[::1]:8890"`

    * `cert_path`: The absolute path to the self-signed certificate generated in step 3 of [Azure Event Grid with MQTT](#2-azure-event-grid-with-mqtt). This file ends in *.cer.

    * `private_key_path`: The absolute path to the private key generated in step 1 of [Azure Event Grid with MQTT](#2-azure-event-grid-with-mqtt). This file ends in *.key.

    * `mqtt_client_id`: The client ID for identifying the MQTT client used in this Cloud Connector. You can keep the default value or change it. The client ID can be any unique value, as long as it's not the same client ID of another client that's publishing to your Azure Event Grid.

    * `mqtt_client_authentication_name`: The client authentication name that you specified in step 1 of [Automated Deployment of Azure Key Vault, Event Grid, and Azure Function App](#automated-deployment-of-azure-key-vault-event-grid-and-azure-function-app), or step 6 of [Azure Event Grid with MQTT](#2-azure-event-grid-with-mqtt) for manual deployment.

    * `event_grid_topic`: The topic that you specified in step 1 of [Automated Deployment of Azure Key Vault, Event Grid, and Azure Function App](#automated-deployment-of-azure-key-vault-event-grid-and-azure-function-app), or step 7 of [Azure Event Grid with MQTT](#2-azure-event-grid-with-mqtt) for manual deployment.

    * `event_grid_namespace_host_name`: The Event Grid Namespace MQTT hostname. You can find the hostname by clicking on your event grid namespace, then copy the MQTT hostname.

## Run

Change directory to the directory with the build artifacts `{freyja-root-dir}/target/debug`. Replace `{freyja-root-dir}` with the repository's root directory.

```shell
cd {freyja-root-dir}/target/debug
./azure-mqtt-connector
```
