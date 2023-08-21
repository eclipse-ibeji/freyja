// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

pub mod azure_cloud_connector_adapter;
mod azure_cloud_connector_adapter_config;

pub use crate::azure_cloud_connector_adapter::AzureCloudConnectorAdapter as CloudAdapterImpl;
