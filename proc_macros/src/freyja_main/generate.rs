// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use proc_macro2::TokenStream;
use quote::quote;

use super::process::FreyjaMainOutput;
use crate::freyja_main::parse::FreyjaMainArgs;

/// Generate code for the use_env! macro
///
/// # Arguments
///
/// - `ir`: the intermediate representation of the output
pub(crate) fn generate(ir: FreyjaMainOutput) -> TokenStream {
    let FreyjaMainOutput {
        args:
            FreyjaMainArgs {
                dt_adapter_type,
                cloud_adapter_type,
                mapping_adapter_type,
                data_adapter_factory_types,
            },
    } = ir;

    quote! {
        #[freyja::export::main]
        async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            use freyja::export::DataAdapterFactory;
            let factories: Vec<Box<dyn DataAdapterFactory + Send + Sync>> = vec![
                #(Box::new(
                    #data_adapter_factory_types::create_new()
                        .expect(concat!("Could not create ", stringify!(#data_adapter_factory_types)))
                )),*
            ];

            freyja::freyja_main::<
                #dt_adapter_type,
                #cloud_adapter_type,
                #mapping_adapter_type
            >(factories)
            .await
        }
    }
}
