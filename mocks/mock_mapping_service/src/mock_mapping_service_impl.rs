// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use log::info;
use mapping_service_proto::v1::{
    mapping_service_server::MappingService, CheckForWorkRequest, CheckForWorkResponse,
    GetMappingRequest, GetMappingResponse,
};
use tonic::{Request, Response, Status};

use crate::MappingState;

/// Implements a Mapping Service
pub struct MockMappingServiceImpl {
    /// The server state
    pub(crate) state: Arc<Mutex<MappingState>>,
}

#[async_trait]
impl MappingService for MockMappingServiceImpl {
    /// Check for any updates to the mapping.
    ///
    /// # Arguments
    /// - `request`: the request
    async fn check_for_work(
        &self,
        _request: Request<CheckForWorkRequest>,
    ) -> Result<Response<CheckForWorkResponse>, Status> {
        info!("Check for work called");
        let mut state = self.state.lock().unwrap();
        let has_work = state.pending_work;

        if has_work {
            info!("Work consumed");
        }

        state.pending_work = false;

        Ok(Response::new(CheckForWorkResponse { has_work }))
    }

    /// Get the latest mapping.
    ///
    /// # Arguments
    /// - `request`: the request
    async fn get_mapping(
        &self,
        _request: Request<GetMappingRequest>,
    ) -> Result<Response<GetMappingResponse>, Status> {
        info!("Get mapping called");
        let state = self.state.lock().unwrap();
        let response = GetMappingResponse {
            mapping: state
                .config
                .values
                .iter()
                .filter_map(|c| {
                    if !state.interactive {
                        Some((c.value.source.clone(), c.value.clone().into()))
                    } else {
                        match c.end {
                            Some(end) if state.count >= c.begin && state.count < end => {
                                Some((c.value.source.clone(), c.value.clone().into()))
                            }
                            None if state.count >= c.begin => {
                                Some((c.value.source.clone(), c.value.clone().into()))
                            }
                            _ => None,
                        }
                    }
                })
                .collect(),
        };

        Ok(Response::new(response))
    }
}
