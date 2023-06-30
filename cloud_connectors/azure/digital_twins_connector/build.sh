#!/bin/bash

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.
# SPDX-License-Identifier: MIT

# Set the current directory to the directory of this script.
cd "$(dirname "$0")"

dotnet build src/core/DigitalTwinsConnector.csproj
dotnet build src/DigitalTwinsClientWrapper/DigitalTwinsClientWrapper.csproj
dotnet build tests/DigitalTwinsClientWrapper.Tests/DigitalTwinsClientWrapper.Tests.csproj