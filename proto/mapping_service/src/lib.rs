// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// Re-export this library so consumers have access to the types used in generation
pub use prost_types;

pub mod v1 {
    use freyja_common::{conversion::Conversion, digital_twin_map_entry::DigitalTwinMapEntry};

    tonic::include_proto!("mapping_service");

    impl From<freyja_common::mapping_adapter::CheckForWorkRequest> for CheckForWorkRequest {
        fn from(_value: freyja_common::mapping_adapter::CheckForWorkRequest) -> Self {
            Self {}
        }
    }

    impl From<CheckForWorkRequest> for freyja_common::mapping_adapter::CheckForWorkRequest {
        fn from(_value: CheckForWorkRequest) -> Self {
            Self {}
        }
    }

    impl From<freyja_common::mapping_adapter::CheckForWorkResponse> for CheckForWorkResponse {
        fn from(value: freyja_common::mapping_adapter::CheckForWorkResponse) -> Self {
            Self {
                has_work: value.has_work,
            }
        }
    }

    impl From<CheckForWorkResponse> for freyja_common::mapping_adapter::CheckForWorkResponse {
        fn from(value: CheckForWorkResponse) -> Self {
            Self {
                has_work: value.has_work,
            }
        }
    }

    impl From<freyja_common::mapping_adapter::GetMappingRequest> for GetMappingRequest {
        fn from(_value: freyja_common::mapping_adapter::GetMappingRequest) -> Self {
            Self {}
        }
    }

    impl From<GetMappingRequest> for freyja_common::mapping_adapter::GetMappingRequest {
        fn from(_value: GetMappingRequest) -> Self {
            Self {}
        }
    }

    impl From<GetMappingResponse> for freyja_common::mapping_adapter::GetMappingResponse {
        fn from(value: GetMappingResponse) -> Self {
            Self {
                map: value
                    .mapping
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect(),
            }
        }
    }

    impl From<freyja_common::mapping_adapter::GetMappingResponse> for GetMappingResponse {
        fn from(value: freyja_common::mapping_adapter::GetMappingResponse) -> Self {
            Self {
                mapping: value.map.into_iter().map(|(k, v)| (k, v.into())).collect(),
            }
        }
    }

    impl From<MapEntry> for DigitalTwinMapEntry {
        fn from(value: MapEntry) -> Self {
            Self {
                source: value.source,
                target: value.target,
                interval_ms: value.interval_ms,
                emit_on_change: value.emit_on_change,
                conversion: value
                    .conversion
                    .map(|c| c.into())
                    .unwrap_or(Conversion::None),
            }
        }
    }

    impl From<DigitalTwinMapEntry> for MapEntry {
        fn from(value: DigitalTwinMapEntry) -> Self {
            Self {
                source: value.source,
                target: value.target,
                interval_ms: value.interval_ms,
                emit_on_change: value.emit_on_change,
                conversion: match value.conversion {
                    Conversion::None => None,
                    Conversion::Linear { mul, offset } => Some(LinearConversion { mul, offset }),
                },
            }
        }
    }

    impl From<LinearConversion> for Conversion {
        fn from(value: LinearConversion) -> Self {
            Self::Linear {
                mul: value.mul,
                offset: value.offset,
            }
        }
    }
}
