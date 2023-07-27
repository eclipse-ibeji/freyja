#!/bin/bash

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.
# SPDX-License-Identifier: MIT

set -e

required_vars=(
    "resource_group"
    "subscription_id"
    "digital_twins_name"
    "thumbprint_of_cert_in_der_format"
    "storage_account_name"
    "function_app_name"
    "key_vault_name"
    "event_grid_topic"
    "event_grid_subscription_name"
    "event_grid_namespace"
    "mqtt_client_authentication_name"
)

parse_config_file() {
    local config_file="$1"
    # Parse the configuration file
    while read -r line; do
        key=$(echo "$line" | sed -e 's/[{}"]//g' | awk -F: '{print $1}')
        value=$(echo "$line" | sed -e 's/[{}"]//g' | awk -F: '{print $2}'| xargs)
        case "$key" in
            resource_group) resource_group="$value" ;;
            subscription_id) subscription_id="$value" ;;
            digital_twins_name) digital_twins_name="$value" ;;
            thumbprint_of_cert_in_der_format) thumbprint_of_cert_in_der_format="$value" ;;
            storage_account_name) storage_account_name="$value" ;;
            function_app_name) function_app_name="$value" ;;
            key_vault_name) key_vault_name="$value" ;;
            event_grid_topic) event_grid_topic="$value" ;;
            event_grid_subscription_name) event_grid_subscription_name="$value" ;;
            event_grid_namespace) event_grid_namespace="$value" ;;
            mqtt_client_authentication_name) mqtt_client_authentication_name="$value" ;;
        esac
    done < <(cat "$config_file" | grep -Eo '"[^"]*"\s*:\s*"[^"]*"')

    # Required values from the mqtt_config.json file


    # Check if all required variables have been set
    missing_vars=()
    for var in "${required_vars[@]}"; do
        if [[ -z "${!var}" ]]; then
            missing_vars+=("$var")
        fi
    done

    # If we have missing key-value pairs, then print all the pairs that are missing from the config file.
    if [[ ${#missing_vars[@]} -gt 0 ]]; then
        echo "Error: Missing required values in config file:"
        for var in "${missing_vars[@]}"; do
            echo "  $var"
        done
        exit 1
    fi
}

# Set the current directory to where the script lives.
cd "$(dirname "$0")"

# Function to display usage information
usage() {
    echo "Usage: $0 [-c|--config-file] <MQTT_CONNECTOR_SETUP_CONFIG_FILE_PATH>"
    echo "       $0 [-r|--resource-group] <RESOURCE_GROUP>"
    echo "          [-s|--subscription-id] <SUBSCRIPTION_ID>"
    echo "          [-dt-n|--digital-twins-name] <DIGITAL_TWINS_NAME>"
    echo "          [-t|--thumbprint-of-cert-in-der-format] <THUMBPRINT_OF_CERT_IN_DER_FORMAT>"
    echo "          [-st-a-n|--storage-account-name] <STORAGE_ACCOUNT_NAME>"
    echo "          [-f-a-n|--function-app-name] <FUNCTION_APP_NAME>"
    echo "          [-k-v-n|--key-vault-name] <KEY_VAULT_NAME>"
    echo "          [-e-g-t|--event-grid-topic] <EVENT_GRID_TOPIC>"
    echo "          [-e-g-s-n|--event-grid-subscription-name] <EVENT_GRID_SUBSCRIPTION_NAME>"
    echo "          [-e-g-n|--event-grid-namespace] <EVENTGRID_NAMESPACE>"
    echo "          [-m-c-a-n|--mqtt-client-authentication-name] <MQTT_CLIENT_AUTHENTICATION_NAME>"
    echo ""
    echo "Example:"
    echo "  $0 -c mqtt_connector_setup.json"
    echo "  $0 -r myResourceGroup -s mySubscriptionId -dt-n myDigitalTwinsName \\"
    echo "     -t myThumbprint -st-a-n myStorageAccountName -f-a-n myFunctionAppName \\"
    echo "     -k-v-n myKeyVaultName -e-g-t myEventGridTopic -e-g-s-n myEventGridSubscriptionName \\"
    echo "     -e-g-n myEventgridNamespace -m-c-a-n myMqttClientAuthenticationName"
}

check_argument_value() {
    if [[ -z "$2" ]]; then
        echo "Error: Missing value for option $1"
        usage
        exit 1
    fi
}

# Function to check if all required arguments have been set
check_required_arguments() {
    # Array to store the names of the missing arguments
    local missing_arguments=()

    # Loop through the array of required argument names
    for arg_name in "${required_vars[@]}"; do
        # Check if the argument value is empty
        if [[ -z "${!arg_name}" ]]; then
            # Add the name of the missing argument to the array
            missing_arguments+=("${arg_name}")
        fi
    done

    # Check if any required argument is missing
    if [[ ${#missing_arguments[@]} -gt 0 ]]; then
        echo -e "\nError: Missing required arguments:"
        printf '  %s\n' "${missing_arguments[@]}"
        [ ! \( \( $# == 1 \) -a \( "$1" == "-c" \) \) ] && echo "  Either provide a config file path or all the arguments, but not both at the same time."
        [ ! \( $# == 22 \) ] && echo "  All arguments must be provided."
        echo ""
        usage
        exit 1
    fi
}

# Parse command line arguments
while [[ $# -gt 0 ]]
do
key="$1"

case $key in
    -c|--config-file)
    config_file="$2"
    parse_config_file "$config_file"
    shift # past argument
    shift # past value
    break # break out of case statement if config file is provided
    ;;
    -r|--resource-group)
    check_argument_value "$@"
    resource_group="$2"
    shift # past argument
    shift # past value
    ;;
    -s|--subscription-id)
    check_argument_value "$@"
    subscription_id="$2"
    shift # past argument
    shift # past value
    ;;
    -dt-n|--digital-twins-name)
    check_argument_value "$@"
    digital_twins_name="$2"
    shift # past argument
    shift # past value
    ;;
    -t|--thumbprint-of-cert-in-der-format)
    check_argument_value "$@"
    thumbprint_of_cert_in_der_format="$2"
    shift # past argument
    shift # past value
    ;;
    -st-a-n|--storage-account-name)
    check_argument_value "$@"
    storage_account_name="$2"
    shift # past argument
    shift # past value
    ;;
    -f-a-n|--function-app-name)
    check_argument_value "$@"
    function_app_name="$2"
    shift # past argument
    shift # past value
    ;;
    -k-v-n|--key-vault-name)
    check_argument_value "$@"
    key_vault_name="$2"
    shift # past argument
    shift # past value
    ;;
    -e-g-t|--event-grid-topic)
    check_argument_value "$@"
    event_grid_topic="$2"
    shift # past argument
    shift # past value
    ;;
    -e-g-s-n|--event-grid-subscription-name)
    check_argument_value "$@"
    event_grid_subscription_name="$2"
    shift # past argument
    shift # past value
    ;;
    -e-g-n|--event-grid-namespace)
    check_argument_value "$@"
    event_grid_namespace="$2"
    shift # past argument
    shift # past value
    ;;
    -m-c-a-n|--mqtt-client-authentication-name)
    check_argument_value "$@"
    mqtt_client_authentication_name="$2"
    shift # past argument
    shift # past value
    ;;
    -h|--help)
    usage
    exit 0
    ;;
    *)
    shift # past argument
    ;;
esac
done

# Check if all required arguments have been set
check_required_arguments

az account set --subscription "$subscription_id"
azure_providers_id_path="/subscriptions/$subscription_id/resourceGroups/$resource_group/providers"

storage_account_query=$(az storage account list --query "[?name=='$storage_account_name']")
if [ "$storage_account_query" == "[]" ]; then
    echo -e "\nCreating an Azure Storage Account"
    az storage account create --name "$storage_account_name" \
        --location westus --resource-group "$resource_group" \
        --sku Standard_LRS --allow-blob-public-access false
else
    echo "Storage Account $storage_account_name already exists."
fi

function_app_query=$(az functionapp list --query "[?name=='$function_app_name']")
if [ "$function_app_query" == "[]" ]; then
    echo -e "\nCreating an Azure Function App"
    az functionapp create --resource-group "$resource_group" \
        --consumption-plan-location westus \
        --runtime dotnet \
        --functions-version 4 \
        --name "$function_app_name" \
        --storage-account "$storage_account_name"
else
    echo "Azure Function App $function_app_name already exists."
fi

# When you create an Azure Function App for the first time, it takes some time to deploy fully.
# Retry publishing the MQTT Connector Function to your Azure Function App.
cd "../mqtt_connector/res/azure_function"
echo -e "\nDeploying Freyja's MQTT Connector Azure Function to $function_app_name"
max_attempts=10
attempt=0
success=false
while [ $attempt -lt $max_attempts ] && ! $success; do
    if func azure functionapp publish "$function_app_name" --csharp; then
        success=true
    else
        echo "Retrying deployment of Freyja's MQTT Connector Azure Function to $function_app_name"
    fi
done
if ! $success; then
    echo "Failed to publish Freyja's MQTT Connector Azure Function after $max_attempts attempts"
    echo "Please try running this script again."
    exit 1
fi
cd "$(dirname "$0")"

# Key Vault
keyvault_query=$(az keyvault list --query "[?name=='$key_vault_name']")
if [ "$keyvault_query" == "[]" ]; then
    echo -e "\nCreating an Azure Key Vault"
    az keyvault create --name "$key_vault_name" --resource-group "$resource_group" --location "westus2"
else
    echo "Key Vault $key_vault_name already exists."
fi
echo -e "\nSetting a secret for ADT-INSTANCE-URL in your Azure Key Vault"
adt_instance_url=$(az dt show --dt-name "$digital_twins_name" -g "$resource_group" --query hostName -o tsv)
az keyvault secret set --name ADT-INSTANCE-URL --vault-name "$key_vault_name" --value "https://$adt_instance_url"

# Event Grid
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

event_grid_subscription_name_query=$(az eventgrid event-subscription list \
    --source-resource-id "$azure_providers_id_path/Microsoft.EventGrid/topics/$event_grid_topic" \
    --query "[?name=='$event_grid_subscription_name']")
if [ "$event_grid_subscription_name_query" == "[]" ]; then
    echo -e "\nCreating Event Grid Subscription"
    az eventgrid event-subscription create --name $event_grid_subscription_name \
        --source-resource-id "$azure_providers_id_path/Microsoft.EventGrid/topics/$event_grid_topic" \
        --endpoint "$azure_providers_id_path/Microsoft.Web/sites/$function_app_name/functions/MQTTConnectorAzureFn" \
        --endpoint-type "azurefunction"
else
    echo "Event Grid Subscription $event_grid_subscription_name already exists."
fi

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
client_properties=$(cat <<EOF
{
    "state": "Enabled",
    "authenticationName": "$mqtt_client_authentication_name",
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
echo -e "\nAssigning a System Managed Identity for $function_app_name"
az webapp identity assign --name $function_app_name --resource-group "$resource_group"

azureFunctionAppObjectID=$(az functionapp identity show --name "$function_app_name" --resource-group "$resource_group" --query "principalId" -o tsv)
echo -e "\nAssigning Key Vault Reader role to $function_app_name"

# When you create an Azure Function App for the first time, it takes some time to deploy fully.
# Retry assigning the Key Vault Reader to your Azure Function App
max_attempts=10
attempt=0
success=false
while [ $attempt -lt $max_attempts ] && ! $success; do
    if az role assignment create --assignee "$azureFunctionAppObjectID" \
        --role "Key Vault Reader" \
        --scope "$azure_providers_id_path/Microsoft.KeyVault/vaults/$key_vault_name"; then

        success=true
    else
        echo "Retrying assigning the Key Vault Reader role to $function_app_name"
    fi
done
if ! $success; then
    echo "Failed to assign the Key Vault Reader role to $function_app_name after $max_attempts attempts"
    echo "Please try running this script again."
    exit 1
fi

# Assigns your Key Vault Access Policies for your Azure Function App.
# Also set the Key Vault setting for access to the ADT-INSTANCE-URL secret in Azure Function App by using the secret identifier.
echo -e "\nSetting KEYVAULT_SETTINGS for the configuration in $function_app_name"
keyVaultSecretURI=$(az keyvault secret show --name "ADT-INSTANCE-URL" --vault-name "$key_vault_name" --query id -o tsv)
az functionapp config appsettings set --name $function_app_name \
    --resource-group "$resource_group" \
    --settings KEYVAULT_SETTINGS="@Microsoft.KeyVault(SecretUri=$keyVaultSecretURI)"
az keyvault set-policy -n $key_vault_name --secret-permissions get --object-id "$azureFunctionAppObjectID"

# Digital Twin system managed identity role assignment for your Azure Function App.
echo -e "\nAssigning Azure Digital Twins Data Owner role to $function_app_name"
az role assignment create --assignee "$azureFunctionAppObjectID" \
    --role "Azure Digital Twins Data Owner" \
    --scope "$azure_providers_id_path/Microsoft.DigitalTwins/digitalTwinsInstances/$digital_twins_name"

echo -e "\nSetup finished for Freyja's Azure MQTT Connector"

echo -e "\n"
echo "Please set the values for mqtt_event_grid_topic and mqtt_client_authentication_name fields of your {freyja-root-dir}/target/debug/mqtt_config.json to the following:"
echo "mqtt_event_grid_topic: $event_grid_topic"
echo "mqtt_client_authentication_name: $mqtt_client_authentication_name"

exit 0