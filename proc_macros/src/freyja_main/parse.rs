// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use proc_macro2::TokenStream;
use syn::bracketed;
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
    pub mapping_adapter_type: Ident,
    pub data_adapter_factory_types: Vec<Ident>,
}

impl Parse for FreyjaMainArgs {
    /// Parses the input stream into `FreyjaMainArgs`
    ///
    /// # Arguments
    ///
    /// - `input`: the input stream
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let dt_adapter_type = input.parse::<Ident>().unwrap();
        let _ = input.parse::<Token![,]>().unwrap();
        let cloud_adapter_type = input.parse::<Ident>().unwrap();
        let _ = input.parse::<Token![,]>().unwrap();
        let mapping_adapter_type = input.parse::<Ident>().unwrap();
        let _ = input.parse::<Token![,]>().unwrap();

        let data_adapter_content;
        let _ = bracketed!(data_adapter_content in input);
        let data_adapter_factory_types =
            Punctuated::<Ident, Token![,]>::parse_terminated(&data_adapter_content)
                .unwrap()
                .into_iter()
                .collect::<Vec<_>>();

        if data_adapter_factory_types.is_empty() {
            panic!("At least one DataAdapterFactory is required");
        }

        let trailing_comma_result = if !input.is_empty() {
            Some(input.parse::<Token![,]>())
        } else {
            None
        };

        if !input.is_empty() || trailing_comma_result.is_some_and(|r| r.is_err()) {
            panic!("Unexpected tokens at end of input");
        }

        Ok(FreyjaMainArgs {
            dt_adapter_type,
            cloud_adapter_type,
            mapping_adapter_type,
            data_adapter_factory_types,
        })
    }
}

#[cfg(test)]
mod freyja_main_parse_tests {
    use quote::{format_ident, quote};
    use std::panic::catch_unwind;

    use super::*;

    #[test]
    fn can_parse_input_in_correct_order() {
        let foo_ident = format_ident!("Foo");
        let bar_ident = format_ident!("Bar");
        let baz_ident = format_ident!("Baz");
        let factory_idents = vec![format_ident!("DA1"), format_ident!("DA2")];
        let factory_idents_clone = factory_idents.clone();

        let input = quote! { #foo_ident, #bar_ident, #baz_ident, [#(#factory_idents),*] };
        let output = parse(input);

        assert_eq!(output.dt_adapter_type, foo_ident);
        assert_eq!(output.cloud_adapter_type, bar_ident);
        assert_eq!(output.mapping_adapter_type, baz_ident);
        for ident in factory_idents.iter() {
            assert!(output.data_adapter_factory_types.contains(ident));
        }

        // Now try a different order
        let input = quote! { #baz_ident, #foo_ident, #bar_ident, [#(#factory_idents_clone),*] };
        let output = parse(input);

        assert_eq!(output.dt_adapter_type, baz_ident);
        assert_eq!(output.cloud_adapter_type, foo_ident);
        assert_eq!(output.mapping_adapter_type, bar_ident);
        for ident in factory_idents {
            assert!(output.data_adapter_factory_types.contains(&ident));
        }
    }

    #[test]
    fn parse_panics_with_invalid_input() {
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

    #[test]
    fn parse_accepts_trailing_comma() {
        let foo_ident = format_ident!("Foo");
        let bar_ident = format_ident!("Bar");
        let baz_ident = format_ident!("Baz");
        let factory_idents = vec![format_ident!("DA1"), format_ident!("DA2")];

        let input = quote! { #foo_ident, #bar_ident, #baz_ident, [#(#factory_idents),*], };
        let result = catch_unwind(|| parse(input));
        assert!(result.is_ok());
    }

    #[test]
    fn parse_panics_with_invalid_trailing_content() {
        let foo_ident = format_ident!("Foo");
        let bar_ident = format_ident!("Bar");
        let baz_ident = format_ident!("Baz");
        let factory_idents = vec![format_ident!("DA1"), format_ident!("DA2")];
        let qux_ident = format_ident!("Qux");

        let input =
            quote! { #foo_ident, #bar_ident, #baz_ident, [#(#factory_idents),*], #qux_ident };
        let result = catch_unwind(|| parse(input));
        assert!(result.is_err());
    }

    #[test]
    fn parse_panics_with_empty_factory_list() {
        let foo_ident = format_ident!("Foo");
        let bar_ident = format_ident!("Bar");
        let baz_ident = format_ident!("Baz");

        let input = quote! { #foo_ident, #bar_ident, #baz_ident, [], };
        let result = catch_unwind(|| parse(input));
        assert!(result.is_err());
    }
}
