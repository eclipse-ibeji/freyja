#!/bin/bash

cd "$(dirname "$0")/.."

DOTNET_DIRECTORY="cloud_connectors/azure/digital_twins_connector"

dotnet tool install --global dotnet-project-licenses
mkdir -p "$DOTNET_DIRECTORY/dotnet_licenses_output"
dotnet-project-licenses -i $DOTNET_DIRECTORY -o -f "$DOTNET_DIRECTORY/dotnet_licenses_output" -u --json -e -c \
--licenseurl-to-license-mappings "$DOTNET_DIRECTORY/license_url_to_type.json"
./tools/dotnet_get_licenses.sh "$DOTNET_DIRECTORY/dotnet_licenses_output/licenses.json" "$DOTNET_DIRECTORY/dotnet_licenses_output"
./tools/dotnet_append_to_notice.sh NOTICE "$DOTNET_DIRECTORY/dotnet_licenses_output/licenses.json"
rm -rf "$DOTNET_DIRECTORY/dotnet_licenses_output"