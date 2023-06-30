// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::{braced, punctuated::Punctuated, Ident, Token, Variant};

/// Parse input for the error! macro
///
/// # Arguments
///
/// - `ts`: the input token stream
pub(crate) fn parse(ts: TokenStream) -> ErrorArgs {
    syn::parse2::<ErrorArgs>(ts).unwrap()
}

/// Arguments to the error macro
#[derive(Debug)]
pub(crate) struct ErrorArgs {
    /// The name for the error type
    pub name: Ident,
    /// The variants that should be defined for the error kind
    pub kinds: Vec<Variant>,
}

impl Parse for ErrorArgs {
    /// Parses the input stream into `ErrorArgs`
    ///
    /// # Arguments
    ///
    /// - `input`: the input stream
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        let content;
        let _ = braced!(content in input);
        let kinds = Punctuated::<Variant, Token![,]>::parse_terminated(&content)?;
        Ok(ErrorArgs {
            name,
            kinds: kinds.into_iter().collect(),
        })
    }
}

#[cfg(test)]
mod error_parse_tests {
    use quote::{format_ident, quote};
    use syn::Fields;

    use super::*;

    fn verify_unit_variant(variant: &Variant, expected_ident: &Ident) {
        assert!(&variant.ident == expected_ident);
        match variant.fields {
            Fields::Unit => {}
            _ => panic!("Expected variant with unit fields"),
        }
    }

    fn verify_tuple_variant(variant: &Variant, expected_ident: &Ident) {
        assert!(&variant.ident == expected_ident);
        match variant.fields {
            Fields::Unnamed(_) => {}
            _ => panic!("Expected variant with tuple fields"),
        }
    }

    fn verify_struct_variant(variant: &Variant, expected_ident: &Ident) {
        assert!(&variant.ident == expected_ident);
        match variant.fields {
            Fields::Named(_) => {}
            _ => panic!("Expected variant with struct fields"),
        }
    }

    #[test]
    fn can_parse_unit_variant() {
        let error = format_ident!("TestError");
        let unit_variant = format_ident!("UnitVariant");

        let input = quote! { #error { #unit_variant } };
        let output = parse(input);

        assert!(output.name == error);
        assert!(output.kinds.len() == 1);
        verify_unit_variant(&output.kinds[0], &unit_variant);
    }

    #[test]
    fn can_parse_tuple_variant() {
        let error = format_ident!("TestError");
        let tuple_variant = format_ident!("TupleVariant");

        let input = quote! { #error { #tuple_variant(u8) } };
        let output = parse(input);

        assert!(output.name == error);
        assert!(output.kinds.len() == 1);
        verify_tuple_variant(&output.kinds[0], &tuple_variant);
    }

    #[test]
    fn can_parse_struct_variant() {
        let error = format_ident!("TestError");
        let struct_variant = format_ident!("StructVariant");

        let input = quote! { #error { #struct_variant { x: u8 } } };
        let output = parse(input);

        assert!(output.name == error);
        assert!(output.kinds.len() == 1);
        verify_struct_variant(&output.kinds[0], &struct_variant);
    }

    #[test]
    fn can_parse_multiple_variants() {
        let error = format_ident!("TestError");
        let unit_variant = format_ident!("TestErrorKindUnit");
        let tuple_variant = format_ident!("TestErrorKindTuple");
        let struct_variant = format_ident!("TestErrorKindStruct");

        let input =
            quote! { TestError { #unit_variant, #tuple_variant(u8), #struct_variant { x: u8 } } };
        let output = parse(input);

        assert!(output.name == error);
        assert!(output.kinds.len() == 3);
        verify_unit_variant(&output.kinds[0], &unit_variant);
        verify_tuple_variant(&output.kinds[1], &tuple_variant);
        verify_struct_variant(&output.kinds[2], &struct_variant);
    }
}
