// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::{punctuated::Punctuated, Ident, Token};

/// Parse input for the freyja_main! macro
///
/// # Arguments
///
/// - `ts`: the input token stream
pub(crate) fn parse(ts: TokenStream) -> FreyjaMainArgs {
    syn::parse2::<FreyjaMainArgs>(ts).unwrap()
}

/// Arguments to the freyja_main macro
#[derive(Debug)]
pub(crate) struct FreyjaMainArgs {
    pub dt_adapter_type: Ident,
    pub cloud_adapter_type: Ident,
    pub mapping_client_type: Ident,
}

impl Parse for FreyjaMainArgs {
    /// Parses the input stream into `FreyjaMainArgs`
    ///
    /// # Arguments
    ///
    /// - `input`: the input stream
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Punctuated::<Ident, Token![,]>::parse_terminated(input)?.into_iter();

        if args.len() != 3 {
            panic!("Expected exactly three arguments to freyja_main");
        }

        Ok(FreyjaMainArgs {
            dt_adapter_type: args.next().unwrap(),
            cloud_adapter_type: args.next().unwrap(),
            mapping_client_type: args.next().unwrap(),
        })
    }
}

#[cfg(test)]
mod use_env_parse_tests {
    use quote::{format_ident, quote};
    use std::panic::catch_unwind;

    use super::*;

    #[test]
    fn can_parse_input_in_correct_order() {
        let foo_ident = format_ident!("Foo");
        let bar_ident = format_ident!("Bar");
        let baz_ident = format_ident!("Baz");

        let input = quote! { #foo_ident, #bar_ident, #baz_ident };
        let output = parse(input);

        assert_eq!(output.dt_adapter_type, foo_ident);
        assert_eq!(output.cloud_adapter_type, bar_ident);
        assert_eq!(output.mapping_client_type, baz_ident);

        // Now try a different order
        let input = quote! { #baz_ident, #foo_ident, #bar_ident };
        let output = parse(input);

        assert_eq!(output.dt_adapter_type, baz_ident);
        assert_eq!(output.cloud_adapter_type, foo_ident);
        assert_eq!(output.mapping_client_type, bar_ident);
    }

    #[test]
    fn parse_panics_with_incorrect_number_of_arguments() {
        let foo_ident = format_ident!("Foo");
        let bar_ident = format_ident!("Bar");
        let baz_ident = format_ident!("Baz");
        let qux_ident = format_ident!("Qux");

        let input = quote! { #foo_ident, #bar_ident };
        let result = catch_unwind(|| parse(input));
        assert!(result.is_err());

        let input = quote! { #foo_ident, #bar_ident, #baz_ident, #qux_ident };
        let result = catch_unwind(|| parse(input));
        assert!(result.is_err());
    }
}
