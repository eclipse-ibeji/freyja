// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod config;
pub mod mock_digital_twin_adapter;

pub use crate::mock_digital_twin_adapter::MockDigitalTwinAdapter as DigitalTwinAdapterImpl;
