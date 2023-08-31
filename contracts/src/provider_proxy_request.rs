// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use tokio::sync::mpsc::UnboundedSender;

use crate::provider_proxy::OperationKind;
pub type Protocol = String;

/// Represents a provider proxy selector request kind
#[derive(Debug)]
pub enum ProviderProxySelectorRequestKind {
    /// Creates or updates a provider's proxy
    CreateOrUpdateProviderProxy {
        /// The id of the entity to link to the proxy
        entity_id: String,
        /// The uri of the provider to proxy
        uri: String,
        /// The communication protocol of the provider to proxy
        protocol: Protocol,
        /// The operation of the provider to proxy
        operation: OperationKind,
    },

    /// Get an entity's value
    GetEntityValue { entity_id: String },
}

/// A client for sending requests to the `ProviderProxySelector`
#[derive(Clone)]
pub struct ProviderProxySelectorRequestSender {
    /// The communication channel for the `ProviderProxySelector`
    tx_provider_proxy_selector_request: UnboundedSender<ProviderProxySelectorRequestKind>,
}

impl ProviderProxySelectorRequestSender {
    /// Creates a provider proxy selector request sender
    /// The tx_provider_proxy_selector_request field is private, so this function is needed to instantiate
    /// this struct
    ///
    /// # Arguments
    /// - `tx_provider_proxy_selector_request`: sends requests to the provider proxy selector
    pub fn new(
        tx_provider_proxy_selector_request: UnboundedSender<ProviderProxySelectorRequestKind>,
    ) -> Self {
        ProviderProxySelectorRequestSender {
            tx_provider_proxy_selector_request,
        }
    }

    /// Sends request to the provider proxy selector
    ///
    /// # Arguments
    /// - `request`: the request to send
    pub fn send_request_to_provider_proxy_selector(
        &self,
        request: ProviderProxySelectorRequestKind,
    ) -> Result<(), ProviderProxySelectorRequestSenderError> {
        self.tx_provider_proxy_selector_request
            .send(request)
            .map_err(ProviderProxySelectorRequestSenderError::receiver_dropped)
    }
}

proc_macros::error! {
    ProviderProxySelectorRequestSenderError {
        ReceiverDropped,
    }
}