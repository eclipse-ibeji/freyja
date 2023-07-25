#!/bin/bash

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.
# SPDX-License-Identifier: MIT

set -e

# Set the current directory to where the script lives.
cd "$(dirname "$0")"

# Function to display usage information
usage() {
    echo "Usage: $0 --resource-group <RESOURCE_GROUP_NAME> --subscription-id <SUBSCRIPTION_ID> --digital-twins-name <DIGITAL_TWINS_RESOURCE_NAME> --cert-thumbprint <THUMBPRINT_OF_CERT_IN_DER_FORMAT>"
}

# Parse command line arguments
while [[ $# -gt 0 ]]
do
key="$1"

case $key in
    --resource-group)
    resource_group="$2"
    shift # past argument
    shift # past value
    ;;
    --subscription-id)
    subscription_id="$2"
    shift # past argument
    shift # past value
    ;;
    --digital-twins-name)
    digital_twins_name="$2"
    shift # past argument
    shift # past value
    ;;
    --cert-thumbprint)
    thumbprint_of_cert_in_der_format="$2"
    shift # past argument
    shift # past value
    ;;
    --help)
    usage
    exit 0
    ;;
esac
done

# Check if all required arguments have been set
if [[ -z "${resource_group}" || -z "${subscription_id}" || -z "${digital_twins_name}" || -z "${thumbprint_of_cert_in_der_format}" ]]; then
    echo "Error: Missing required arguments:"
    [[ -z "${resource_group}" ]] && echo "  --resource-group"
    [[ -z "${subscription_id}" ]] && echo "  --subscription-id"
    [[ -z "${digital_twins_name}" ]] && echo "  --digital-twins-name"
    [[ -z "${thumbprint_of_cert_in_der_format}" ]] && echo "  --cert-thumbprint"
    echo -e "\n"
    usage
    exit 1
fi

az account set --subscription "$subscription_id"
azure_providers_id_path="/subscriptions/$subscription_id/resourceGroups/$resource_group/providers"

read -p "Enter the Azure Storage Account name to use: " storage_account_name
storage_account_query=$(az storage account list --query "[?name=='$storage_account_name']")
if [ "$storage_account_query" == "[]" ]; then
    echo -e "\nCreating an Azure Storage Account"
    az storage account create --name "$storage_account_name" \
        --location westus --resource-group "$resource_group" \
        --sku Standard_LRS --allow-blob-public-access false
else
    echo "Storage Account $storage_account_name already exists."
fi

read -p "Enter the Azure Function App name to use: " azure_function_app_name
function_app_query=$(az functionapp list --query "[?name=='$azure_function_app_name']")
if [ "$function_app_query" == "[]" ]; then
    echo -e "\nCreating an Azure Function App"
    az functionapp create --resource-group "$resource_group" \
        --consumption-plan-location westus \
        --runtime dotnet \
        --functions-version 4 \
        --name "$azure_function_app_name" \
        --storage-account "$storage_account_name"
else
    echo "Azure Function App $azure_function_app_name already exists."
fi

# When you create an Azure Function App for the first time, it takes some time to deploy fully.
# Retry publishing the MQTT Connector Function to your Azure Function App.
cd "../mqtt_connector/res/azure_function"
echo -e "\nDeploying Freyja's MQTT Connector Azure Function to $azure_function_app_name"
max_attempts=10
attempt=0
success=false
while [ $attempt -lt $max_attempts ] && ! $success; do
    if func azure functionapp publish "$azure_function_app_name" --csharp; then
        success=true
    else
        echo "Retrying deployment of Freyja's MQTT Connector Azure Function to $azure_function_app_name"
    fi
done
if ! $success; then
    echo "Failed to publish Freyja's MQTT Connector Azure Function after $max_attempts attempts"
    echo "Please try running this script again."
    exit 1
fi
cd "$(dirname "$0")"

# Key Vault
read -p "Enter the Azure Key Vault name to use: " keyvault_name
keyvault_query=$(az keyvault list --query "[?name=='$keyvault_name']")
if [ "$keyvault_query" == "[]" ]; then
    echo -e "\nCreating an Azure Key Vault"
    az keyvault create --name "$keyvault_name" --resource-group "$resource_group" --location "westus2"
else
    echo "Key Vault $keyvault_name already exists."
fi
echo -e "\nSetting a secret for ADT-INSTANCE-URL in your Azure Key Vault"
adt_instance_url=$(az dt show --dt-name "$digital_twins_name" -g "$resource_group" --query hostName -o tsv)
az keyvault secret set --name ADT-INSTANCE-URL --vault-name "$keyvault_name" --value "https://$adt_instance_url"

# Event Grid
read -p "Enter the Event Grid Topic name to use: " event_grid_topic
event_grid_topic_query=$(az eventgrid topic list --resource-group "$resource_group" --query "[?name=='$event_grid_topic']")
if [ "$event_grid_topic_query" == "[]" ]; then
    echo -e "\nCreating the event grid topic '$event_grid_topic'"
    az eventgrid topic create --name "$event_grid_topic" -l westus2 -g "$resource_group" --input-schema cloudeventschemav1_0
else
    echo "Event Grid topic $event_grid_topic already exists."
fi

echo -e "\nAssigning EventGrid Data Sender Role"
# Gets the signed-in Azure CLI user's object ID
userObjectID=$(az ad signed-in-user show --query id -o tsv)
az role assignment create --assignee "$userObjectID" --role "EventGrid Data Sender" \
    --scope "$azure_providers_id_path/Microsoft.EventGrid/topics/$event_grid_topic"

read -p "Enter the Event Grid Subscription name to use: " event_grid_subscription
event_grid_subscription_query=$(az eventgrid event-subscription list \
    --source-resource-id "$azure_providers_id_path/Microsoft.EventGrid/topics/$event_grid_topic" \
    --query "[?name=='$event_grid_subscription']")
if [ "$event_grid_subscription_query" == "[]" ]; then
    echo -e "\nCreating Event Grid Subscription"
    az eventgrid event-subscription create --name $event_grid_subscription \
        --source-resource-id "$azure_providers_id_path/Microsoft.EventGrid/topics/$event_grid_topic" \
        --endpoint "$azure_providers_id_path/Microsoft.Web/sites/$azure_function_app_name/functions/MQTTConnectorAzureFn" \
        --endpoint-type "azurefunction"
else
    echo "Event Grid Subscription $event_grid_subscription already exists."
fi

read -p "Enter the Event Grid Namespace name to use: " event_grid_namespace
event_grid_namespace_query=$(az resource list --resource-group "$resource_group" \
    --resource-type "Microsoft.EventGrid/namespaces" \
    --query "[?name=='$event_grid_namespace']")
namespace_properties=$(cat <<EOF
{
    "properties": {
        "inputSchema": "CloudEventSchemaV1_0",
        "topicSpacesConfiguration": {
            "state": "Enabled",
            "routeTopicResourceId": "$azure_providers_id_path/Microsoft.EventGrid/topics/$event_grid_topic"
        },
        "isZoneRedundant": true
    },
    "location": "westus2"
}
EOF
)
if [ "$event_grid_namespace_query" == "[]" ]; then
    echo -e "\nCreating an Event Grid Namespace"
    az resource create --resource-type Microsoft.EventGrid/namespaces \
        --id "$azure_providers_id_path/Microsoft.EventGrid/namespaces/$event_grid_namespace" \
        --is-full-object \
        --api-version 2023-06-01-preview \
        --properties "$namespace_properties"
else
    echo "Event Grid Namespace $event_grid_namespace already exists."
fi

echo -e "\nCreating an Event Grid Client"
read -p "Enter the Event Grid Client Authentication name to use: " client_authentication_name
client_properties=$(cat <<EOF
{
    "state": "Enabled",
    "authenticationName": "$client_authentication_name",
    "clientCertificateAuthentication": {
        "validationScheme": "ThumbprintMatch",
        "allowedThumbprints": [
            "$thumbprint_of_cert_in_der_format"
        ]
    }
}
EOF
)
az resource create --resource-type Microsoft.EventGrid/namespaces/clients \
    --id "$azure_providers_id_path/Microsoft.EventGrid/namespaces/$event_grid_namespace/clients/vehicle1" \
    --api-version 2023-06-01-preview \
    --properties "$client_properties"

echo -e "\nCreating an Event Grid Topic Space"
topic_space_properties=$(cat <<EOF
{
    "topicTemplates": [
        "$event_grid_topic"
    ]
}
EOF
)
az resource create --resource-type Microsoft.EventGrid/namespaces/topicSpaces \
    --id "$azure_providers_id_path/Microsoft.EventGrid/namespaces/$event_grid_namespace/topicSpaces/vehicles" \
    --api-version 2023-06-01-preview \
    --properties "$topic_space_properties"
echo "Replace the mqtt_event_grid_topic field of your mqtt_config.json with "$event_grid_topic""

echo -e "\nCreating Event Grid Permission Binding for Publisher"
permission_binding_properties='{
    "clientGroupName": "$all",
    "permission": "Publisher",
    "topicSpaceName": "vehicles"
}'
az resource create --resource-type Microsoft.EventGrid/namespaces/permissionBindings \
    --id "$azure_providers_id_path/Microsoft.EventGrid/namespaces/$event_grid_namespace/permissionBindings/vehicle1" \
    --api-version 2023-06-01-preview \
    --properties "$permission_binding_properties"

# Assigns your Azure Function App's managed system identity permissions
echo -e "\nAssigning a System Managed Identity for $azure_function_app_name"
az webapp identity assign --name $azure_function_app_name --resource-group "$resource_group"

azureFunctionAppObjectID=$(az functionapp identity show --name "$azure_function_app_name" --resource-group "$resource_group" --query "principalId" -o tsv)
echo -e "\nAssigning Key Vault Reader role to $azure_function_app_name"
az role assignment create --assignee "$azureFunctionAppObjectID" \
    --role "Key Vault Reader" \
    --scope "$azure_providers_id_path/Microsoft.KeyVault/vaults/$keyvault_name"

# Assigns your Key Vault Access Policies for your Azure Function App.
# Also set the Key Vault setting for access to the ADT-INSTANCE-URL secret in Azure Function App by using the secret identifier.
echo -e "\nSetting KEYVAULT_SETTINGS for the configuration in $azure_function_app_name"
keyVaultSecretURI=$(az keyvault secret show --name "ADT-INSTANCE-URL" --vault-name "$keyvault_name" --query id -o tsv)
az functionapp config appsettings set --name $azure_function_app_name \
    --resource-group "$resource_group" \
    --settings KEYVAULT_SETTINGS="@Microsoft.KeyVault(SecretUri=$keyVaultSecretURI)"
az keyvault set-policy -n $keyvault_name --secret-permissions get --object-id "$azureFunctionAppObjectID"

# Digital Twin system managed identity role assignment for your Azure Function App.
echo -e "\nAssigning Azure Digital Twins Data Owner role to $azure_function_app_name"
az role assignment create --assignee "$azureFunctionAppObjectID" \
    --role "Azure Digital Twins Data Owner" \
    --scope "$azure_providers_id_path/Microsoft.DigitalTwins/digitalTwinsInstances/$digital_twins_name"

echo -e "\nSetup finished for Freyja's Azure MQTT Connector"

echo -e "\n"
echo "Please set the values for mqtt_event_grid_topic and mqtt_client_authentication_name fields of your {freyja-root-dir}/target/debug/mqtt_config.json to the following:"
echo "mqtt_event_grid_topic: $event_grid_topic"
echo "mqtt_client_authentication_name: $client_authentication_name"

exit 0