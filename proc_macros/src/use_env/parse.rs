// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::{punctuated::Punctuated, Ident, Token};

/// Parse input for the use_env! macro
///
/// # Arguments
///
/// - `ts`: the input token stream
pub(crate) fn parse(ts: TokenStream) -> UseEnvArgs {
    syn::parse2::<UseEnvArgs>(ts).unwrap()
}

/// Arguments to the use_env macro
#[derive(Debug)]
pub(crate) struct UseEnvArgs {
    /// The list of arguments
    pub args: Vec<UseEnvArg>,
}

/// An argument to the use_env macro
#[derive(Debug)]
pub(crate) struct UseEnvArg {
    /// The environment variable to look up
    pub env: Ident,
    /// The thing to import from the looked up namespace
    pub target_type: Ident,
}

impl Parse for UseEnvArgs {
    /// Parses the input stream into `UseEnvArgs`
    ///
    /// # Arguments
    ///
    /// - `input`: the input stream
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args = Punctuated::<UseEnvArg, Token![,]>::parse_terminated(input)?
            .into_iter()
            .collect();
        Ok(UseEnvArgs { args })
    }
}

impl Parse for UseEnvArg {
    /// Parses the input stream a `UseEnvArg`
    ///
    /// # Arguments
    ///
    /// - `input`: the input stream
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let env = input.parse::<Ident>()?;
        input.parse::<Token![::]>()?;
        let target_type = input.parse::<Ident>()?;
        Ok(UseEnvArg { env, target_type })
    }
}

#[cfg(test)]
mod use_env_parse_tests {
    use quote::{format_ident, quote};

    use super::*;

    #[test]
    fn can_parse_single_arg() {
        let env_ident = format_ident!("USE_ENV_TEST");
        let target_type = format_ident!("TargetType");

        let input = quote! { #env_ident::#target_type };
        let output = parse(input);

        assert!(output.args.len() == 1);
        assert!(output.args[0].env == env_ident);
        assert!(output.args[0].target_type == target_type);
    }

    #[test]
    fn can_parse_mutiple_args() {
        let env_ident1 = format_ident!("USE_ENV_TEST_1");
        let env_ident2 = format_ident!("USE_ENV_TEST_2");
        let target_type1 = format_ident!("TargetType1");
        let target_type2 = format_ident!("TargetType2");

        let input = quote! { #env_ident1::#target_type1, #env_ident2::#target_type2 };
        let output = parse(input);

        assert!(output.args.len() == 2);
        assert!(output.args[0].env == env_ident1);
        assert!(output.args[0].target_type == target_type1);
        assert!(output.args[1].env == env_ident2);
        assert!(output.args[1].target_type == target_type2);
    }
}
