// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, Expr, FnArg, ItemEnum, ItemImpl, ItemStruct};

use super::process::*;

/// Generate code for the error! macro
///
/// # Arguments
///
/// - `ts`: the input token stream
pub(crate) fn generate(ir: ErrorOutput) -> TokenStream {
    quote!(#ir)
}

impl ToTokens for ErrorOutput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.kind.to_tokens(tokens);
        self.error_struct.to_tokens(tokens);
        self.self_impl.to_tokens(tokens);
        self.display_impl.to_tokens(tokens);
        self.error_impl.to_tokens(tokens);
        self.from_kind_impl.to_tokens(tokens);
    }
}

impl ToTokens for ErrorKindDef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.error_kind_name;
        let variants = &self.variants;

        let kind_enum: ItemEnum = parse_quote! {
            // Ignore this warning because we want to be able to support non-Eq variants but detecting when to derive Eq is difficult
            #[allow(clippy::derive_partial_eq_without_eq)]
            #[derive(Debug, PartialEq, Clone)]
            pub enum #name {
                #(#variants),*
            }
        };

        kind_enum.to_tokens(tokens);
    }
}

impl ToTokens for ErrorStructDef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.error_name;
        let kind = &self.kind_name;

        let error_struct: ItemStruct = parse_quote! {
            #[derive(Debug)]
            pub struct #name {
                kind: #kind,
                inner: Option<Box<dyn std::error::Error + Send + Sync>>,
            }
        };

        error_struct.to_tokens(tokens);
    }
}

impl ToTokens for ErrorStructImplDef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.error_name;
        let kind = &self.kind_name;

        let mut self_impl: ItemImpl = parse_quote! {
            impl #name {
                pub fn kind(&self) -> #kind {
                    self.kind.clone()
                }

                pub fn new(kind: #kind) -> Self {
                    Self { kind, inner: None }
                }
            }
        };

        self_impl.items.append(
            &mut self.ctors.iter()
                .map(|c| {
                    let fn_name = &c.fn_name;
                    let variant = &c.variant_name;
                    let ctor_additional_args: Vec<FnArg> = c.fn_args.iter()
                        .map(|a| {
                            let arg_name = &a.arg_name;
                            let arg_type = &a.arg_type;
                            parse_quote!(#arg_name: #arg_type)
                        })
                        .collect();
                    let args = c.fn_args.iter()
                        .map(|a| &a.arg_name)
                        .collect::<Vec<_>>();
                    let variant_ctor: Expr = match c.variant_style {
                        VariantStyle::Unit => parse_quote!(#kind::#variant),
                        VariantStyle::Tuple => parse_quote!(#kind::#variant(#(#args),*)),
                        VariantStyle::Struct => parse_quote!(#kind::#variant{#(#args),*}),
                    };

                    parse_quote! {
                        pub fn #fn_name<E: Into<Box<dyn std::error::Error + Send + Sync>>>(error: E #(, #ctor_additional_args)*) -> Self {
                            Self { kind: #variant_ctor, inner: Some(error.into()) }
                        }
                    }
                })
                .collect::<Vec<_>>()
            );

        self_impl.to_tokens(tokens);
    }
}

impl ToTokens for DisplayImplDef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let error = &self.error_name;
        let error_as_string = error.to_string();

        let display_impl: ItemImpl = parse_quote! {
            impl std::fmt::Display for #error {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}: {:?}", #error_as_string, self.kind)
                }
            }
        };

        display_impl.to_tokens(tokens);
    }
}

impl ToTokens for ErrorImplDef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let error = &self.error_name;

        let error_impl: ItemImpl = parse_quote! {
            impl std::error::Error for #error {
                fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                    self.inner.as_ref().map(|i| &**i as _)
                }
            }
        };

        error_impl.to_tokens(tokens);
    }
}

impl ToTokens for FromKindImplDef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let error = &self.error_name;
        let kind = &self.error_kind_name;

        let from_kind_impl: ItemImpl = parse_quote! {
            impl From<#kind> for #error {
                fn from(value: #kind) -> Self {
                    Self::new(value)
                }
            }
        };

        from_kind_impl.to_tokens(tokens);
    }
}

#[cfg(test)]
mod error_generate_tests {
    use quote::format_ident;
    use syn::{
        parse::Parse, punctuated::Punctuated, Attribute, Block, Fields, Ident, Meta, Token,
        Visibility,
    };

    use super::*;

    fn get_test_input(error_name: Ident, error_kind_name: Ident) -> ErrorOutput {
        ErrorOutput {
            kind: ErrorKindDef {
                error_kind_name: error_kind_name.clone(),
                variants: vec![
                    parse_quote!(UnitVariant),
                    parse_quote!(TupleVariant(u8)),
                    parse_quote!(StructVariant { x: u8 }),
                ],
            },
            error_struct: ErrorStructDef {
                error_name: error_name.clone(),
                kind_name: error_kind_name.clone(),
            },
            self_impl: ErrorStructImplDef {
                error_name: error_name.clone(),
                kind_name: error_kind_name.clone(),
                ctors: vec![
                    ErrorCtorDef {
                        variant_name: format_ident!("UnitVariant"),
                        fn_name: format_ident!("unit_variant"),
                        variant_style: VariantStyle::Unit,
                        fn_args: vec![],
                    },
                    ErrorCtorDef {
                        variant_name: format_ident!("TupleVariant"),
                        fn_name: format_ident!("tuple_variant"),
                        variant_style: VariantStyle::Tuple,
                        fn_args: vec![ErrorCtorArgDef {
                            arg_name: format_ident!("_0"),
                            arg_type: parse_quote!(u8),
                        }],
                    },
                    ErrorCtorDef {
                        variant_name: format_ident!("StructVariant"),
                        fn_name: format_ident!("struct_variant"),
                        variant_style: VariantStyle::Struct,
                        fn_args: vec![ErrorCtorArgDef {
                            arg_name: format_ident!("x"),
                            arg_type: parse_quote!(u8),
                        }],
                    },
                ],
            },
            display_impl: DisplayImplDef {
                error_name: error_name.clone(),
            },
            error_impl: ErrorImplDef {
                error_name: error_name.clone(),
            },
            from_kind_impl: FromKindImplDef {
                error_name,
                error_kind_name,
            },
        }
    }

    fn verify_attribute_macro_argument_list(
        input: Vec<Attribute>,
        macro_name: &str,
        expected_args: &[&str],
    ) {
        struct DeriveArgs {
            value: Vec<String>,
        }

        impl Parse for DeriveArgs {
            fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                let result = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
                Ok(DeriveArgs {
                    value: result.into_iter().map(|i| i.to_string()).collect(),
                })
            }
        }

        let derive_tokens = input.iter().find_map(|a| match a.meta {
            Meta::List(ref l) => {
                if l.path.segments[0].ident == macro_name {
                    Some(l.tokens.clone())
                } else {
                    None
                }
            }
            _ => None,
        });

        assert!(
            derive_tokens.is_some(),
            "Expected the {} attribute with an argument list",
            macro_name
        );

        let derive_args = syn::parse2::<DeriveArgs>(derive_tokens.unwrap()).unwrap();
        assert!(expected_args
            .iter()
            .all(|s| derive_args.value.contains(&s.to_string())));
    }

    #[test]
    fn output_is_valid_rust_code() {
        let error_name = format_ident!("TestError");
        let error_kind_name = format_ident!("{}Kind", error_name);
        let ir = get_test_input(error_name, error_kind_name);

        let output = generate(ir);

        // Wrap the output in a block and just make sure we can parse it at all
        let output_block = quote!({#output});
        assert!(syn::parse2::<Block>(output_block).is_ok());
    }

    #[test]
    fn error_kind_enum_is_correct() {
        let error_name = format_ident!("TestError");
        let error_kind_name = format_ident!("{}Kind", error_name);
        let ir = get_test_input(error_name, error_kind_name).kind;
        let item: ItemEnum = parse_quote!(#ir);

        // Verify derive macro
        verify_attribute_macro_argument_list(
            item.attrs,
            "derive",
            &["Debug", "PartialEq", "Clone"],
        );

        // Verify visibility
        match item.vis {
            Visibility::Public(_) => {}
            _ => panic!("Expected public visibility"),
        };

        // Verify number of variants. Content of variants should be covered by process tests
        assert_eq!(item.variants.len(), ir.variants.len());
    }

    #[test]
    fn error_struct_is_correct() {
        let error_name = format_ident!("TestError");
        let error_kind_name = format_ident!("{}Kind", error_name);
        let ir = get_test_input(error_name, error_kind_name).error_struct;
        let item: ItemStruct = parse_quote!(#ir);

        // Verify derive macro
        verify_attribute_macro_argument_list(item.attrs, "derive", &["Debug"]);

        // Verify visibility
        match item.vis {
            Visibility::Public(_) => {}
            _ => panic!("Expected public visibility"),
        };

        // Verify fields
        match item.fields {
            Fields::Named(ref fields) => {
                // Just validating the presence of the fields, checking the types is very complex
                assert!(fields
                    .named
                    .iter()
                    .any(|f| f.ident.as_ref().unwrap() == "kind"));
                assert!(fields
                    .named
                    .iter()
                    .any(|f| f.ident.as_ref().unwrap() == "inner"));
            }
            _ => panic!("Expected named struct fields"),
        }
    }

    #[test]
    fn error_struct_impl_is_correct() {
        let error_name = format_ident!("TestError");
        let error_kind_name = format_ident!("{}Kind", error_name);
        let ir = get_test_input(error_name, error_kind_name).self_impl;

        // Due to complexity, this only verifies that we can parse the generated code into an impl
        let tokens = quote!(#ir);
        assert!(syn::parse2::<ItemImpl>(tokens).is_ok());
    }

    #[test]
    fn display_impl_is_correct() {
        let error_name = format_ident!("TestError");
        let error_kind_name = format_ident!("{}Kind", error_name);
        let ir = get_test_input(error_name, error_kind_name).display_impl;

        // Due to complexity, this only verifies that we can parse the generated code into an impl
        let tokens = quote!(#ir);
        assert!(syn::parse2::<ItemImpl>(tokens).is_ok());
    }

    #[test]
    fn error_impl_is_correct() {
        let error_name = format_ident!("TestError");
        let error_kind_name = format_ident!("{}Kind", error_name);
        let ir = get_test_input(error_name, error_kind_name).error_impl;

        // Due to complexity, this only verifies that we can parse the generated code into an impl
        let tokens = quote!(#ir);
        assert!(syn::parse2::<ItemImpl>(tokens).is_ok());
    }

    #[test]
    fn from_kind_impl_is_correct() {
        let error_name = format_ident!("TestError");
        let error_kind_name = format_ident!("{}Kind", error_name);
        let ir = get_test_input(error_name, error_kind_name).from_kind_impl;

        // Due to complexity, this only verifies that we can parse the generated code into an impl
        let tokens = quote!(#ir);
        assert!(syn::parse2::<ItemImpl>(tokens).is_ok());
    }
}
