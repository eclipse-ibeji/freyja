// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, Stmt};

use super::process::{UseEnvOutput, UseStmt};

/// Generate code for the use_env! macro
///
/// # Arguments
///
/// - `ir`: the intermediate representation of the output
pub(crate) fn generate(ir: UseEnvOutput) -> TokenStream {
    let use_stmts = ir.use_stmts;

    quote!(#(#use_stmts)*)
}

impl ToTokens for UseStmt {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let pieces = &self.pieces;
        let stmt: Stmt = parse_quote!(use #(#pieces)::* ;);
        stmt.to_tokens(tokens);
    }
}

#[cfg(test)]
mod use_env_generate_tests {
    use quote::format_ident;
    use syn::Item;

    use super::*;

    #[test]
    fn output_is_valid_use_statement() {
        let input = UseEnvOutput {
            use_stmts: vec![UseStmt {
                pieces: vec![
                    format_ident!("target_crate1"),
                    format_ident!("target_type1"),
                ],
            }],
        };

        let output = generate(input);

        let parse_result = syn::parse2::<Stmt>(output);
        assert!(parse_result.is_ok());

        match parse_result.unwrap() {
            Stmt::Item(Item::Use(_)) => {}
            _ => panic!("Expected a use statement"),
        }
    }
}
