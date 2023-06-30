// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use convert_case::{Case, Casing};
use quote::format_ident;
use syn::{Fields, Ident, Type, Variant};

use super::parse::ErrorArgs;

/// Process data for the use_env! macro
///
/// # Arguments
///
/// - `ts`: the input token stream
pub(crate) fn process(args: ErrorArgs) -> ErrorOutput {
    let error_kind_name = format_ident!("{}Kind", args.name);

    let self_impl = ErrorStructImplDef {
        error_name: args.name.clone(),
        kind_name: error_kind_name.clone(),
        ctors: args.kinds.iter().map(create_variant_ctor).collect(),
    };

    let kind = ErrorKindDef {
        error_kind_name: error_kind_name.clone(),
        variants: args.kinds,
    };

    let error_struct = ErrorStructDef {
        error_name: args.name.clone(),
        kind_name: error_kind_name.clone(),
    };

    let display_impl = DisplayImplDef {
        error_name: args.name.clone(),
    };

    let error_impl = ErrorImplDef {
        error_name: args.name.clone(),
    };

    let from_kind_impl = FromKindImplDef {
        error_name: args.name,
        error_kind_name,
    };

    ErrorOutput {
        kind,
        error_struct,
        self_impl,
        display_impl,
        error_impl,
        from_kind_impl,
    }
}

/// Helper to generate a constructor for a specific variant
///
/// # Arguments
///
/// - `variant`: the variant to generate the constructor from
fn create_variant_ctor(variant: &Variant) -> ErrorCtorDef {
    ErrorCtorDef {
        fn_name: format_ident!("{}", variant.ident.to_string().to_case(Case::Snake)),
        fn_args: create_variant_args(variant),
        variant_name: variant.ident.clone(),
        variant_style: match variant.fields {
            Fields::Unit => VariantStyle::Unit,
            Fields::Unnamed(_) => VariantStyle::Tuple,
            Fields::Named(_) => VariantStyle::Struct,
        },
    }
}

/// Helper to generate arguments for a specific variant constructor
///
/// # Arguments
///
/// - `variant`: the variant to generate the constructor from
fn create_variant_args(variant: &Variant) -> Vec<ErrorCtorArgDef> {
    match variant.fields {
        Fields::Unit => Vec::new(),
        Fields::Unnamed(ref fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, f)| ErrorCtorArgDef {
                arg_name: format_ident!("_{i}"),
                arg_type: f.ty.clone(),
            })
            .collect(),
        Fields::Named(ref fields) => fields
            .named
            .iter()
            .map(|f| ErrorCtorArgDef {
                arg_name: f.ident.clone().unwrap(),
                arg_type: f.ty.clone(),
            })
            .collect(),
    }
}

/// An intermediate representation of the output of the error! macro
#[derive(Debug)]
pub(crate) struct ErrorOutput {
    /// The definition of the error kind enum
    pub kind: ErrorKindDef,

    /// The definition of the error struct
    pub error_struct: ErrorStructDef,

    /// The definition of the error struct's implementation
    pub self_impl: ErrorStructImplDef,

    /// The definition of the error struct's implementation of the Display trait
    pub display_impl: DisplayImplDef,

    /// The definition of the error struct's implementation of the Error trait
    pub error_impl: ErrorImplDef,

    /// The definition of the error struct's implementation of the From<ErrorKind> trait
    pub from_kind_impl: FromKindImplDef,
}

/// Defines the error kind enum
#[derive(Debug)]
pub(crate) struct ErrorKindDef {
    /// The name of the enum
    pub error_kind_name: Ident,

    /// The enum variants
    pub variants: Vec<Variant>,
}

/// Defines the error struct
#[derive(Debug)]
pub(crate) struct ErrorStructDef {
    /// The name of the struct
    pub error_name: Ident,

    /// The name of the associated error kind enum
    pub kind_name: Ident,
}

/// Defines the error struct's implementation
#[derive(Debug)]
pub(crate) struct ErrorStructImplDef {
    /// The name of the error type
    pub error_name: Ident,

    /// The name of the associated error kind enum
    pub kind_name: Ident,

    /// The list of constructors that should be generated in addition to the default functions
    pub ctors: Vec<ErrorCtorDef>,
}

/// Defines a constructor function for the error struct
#[derive(Debug)]
pub(crate) struct ErrorCtorDef {
    /// The name of the function
    pub fn_name: Ident,

    /// The function's arguments
    pub fn_args: Vec<ErrorCtorArgDef>,

    /// The name of the error kind variant that will be constructed
    pub variant_name: Ident,

    /// The style of the error kind variant that will be constructed
    pub variant_style: VariantStyle,
}

/// Defines an argument for an error constructor
#[derive(Debug)]
pub(crate) struct ErrorCtorArgDef {
    /// The argument name
    pub arg_name: Ident,

    /// The argument type
    pub arg_type: Type,
}

/// Defines an enum variant style
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum VariantStyle {
    /// A unit variant (e.g. `Foo`)
    Unit,

    /// A tuple-like variant (e.g. `Bar(u8)`)
    Tuple,

    /// A struct-like variant (e.g. `Baz { b: u8, a: u8, z: u8 }`)
    Struct,
}

/// Defines the error struct's implementation of the Display trait
#[derive(Debug)]
pub(crate) struct DisplayImplDef {
    /// The name of the error type
    pub error_name: Ident,
}

/// Defines the error struct's implementation of the Error trait
#[derive(Debug)]
pub(crate) struct ErrorImplDef {
    /// The name of the error type
    pub error_name: Ident,
}

/// Defines the error struct's implementation of the From<ErrorKind> trait
#[derive(Debug)]
pub(crate) struct FromKindImplDef {
    /// The name of the error type
    pub error_name: Ident,
    /// The name of the error kind type
    pub error_kind_name: Ident,
}

#[cfg(test)]
mod error_process_tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn can_generate_error_type() {
        let error_name = format_ident!("TestError");
        let input = ErrorArgs {
            name: error_name.clone(),
            kinds: vec![
                parse_quote!(UnitVariant),
                parse_quote!(TupleVariant(u8)),
                parse_quote!(StructVariant { x: u8 }),
            ],
        };

        let output = process(input);

        let expected_kind = format_ident!("{}Kind", error_name);
        verify_kind(
            &output.kind,
            ErrorKindDef {
                error_kind_name: expected_kind.clone(),
                variants: vec![
                    parse_quote!(UnitVariant),
                    parse_quote!(TupleVariant(u8)),
                    parse_quote!(StructVariant { x: u8 }),
                ],
            },
        );

        verify_error_struct(
            &output.error_struct,
            ErrorStructDef {
                error_name: error_name.clone(),
                kind_name: expected_kind.clone(),
            },
        );

        verify_self_impl(
            &output.self_impl,
            ErrorStructImplDef {
                error_name: error_name.clone(),
                kind_name: expected_kind.clone(),
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
        );

        verify_display_impl(
            &output.display_impl,
            DisplayImplDef {
                error_name: error_name.clone(),
            },
        );

        verify_error_impl(
            &output.error_impl,
            ErrorImplDef {
                error_name: error_name.clone(),
            },
        );

        verify_from_kind_impl(
            &output.from_kind_impl,
            FromKindImplDef {
                error_name,
                error_kind_name: expected_kind,
            },
        );
    }

    fn verify_kind(actual: &ErrorKindDef, expected: ErrorKindDef) {
        assert_eq!(actual.error_kind_name, expected.error_kind_name);
        assert_eq!(actual.variants.len(), expected.variants.len());

        for i in 0..expected.variants.len() {
            let actual_variant = actual
                .variants
                .iter()
                .find(|c| c.ident == expected.variants[i].ident);
            assert!(actual_variant.is_some());
            assert_eq!(*actual_variant.unwrap(), expected.variants[i]);
        }
    }

    fn verify_error_struct(actual: &ErrorStructDef, expected: ErrorStructDef) {
        assert_eq!(actual.error_name, expected.error_name);
        assert_eq!(actual.kind_name, expected.kind_name);
    }

    fn verify_self_impl(actual: &ErrorStructImplDef, expected: ErrorStructImplDef) {
        assert_eq!(actual.error_name, expected.error_name);
        assert_eq!(actual.kind_name, expected.kind_name);
        assert_eq!(actual.ctors.len(), expected.ctors.len());

        for i in 0..expected.ctors.len() {
            let actual_ctor = actual
                .ctors
                .iter()
                .find(|c| c.fn_name == expected.ctors[i].fn_name);
            assert!(actual_ctor.is_some());
            verify_ctor(actual_ctor.unwrap(), &expected.ctors[i]);
        }
    }

    fn verify_ctor(actual: &ErrorCtorDef, expected: &ErrorCtorDef) {
        assert_eq!(actual.fn_name, expected.fn_name);
        assert_eq!(actual.variant_style, expected.variant_style);
        assert_eq!(actual.fn_args.len(), expected.fn_args.len());

        // What if they aren't in the same order?
        for i in 0..expected.fn_args.len() {
            let actual_arg = actual
                .fn_args
                .iter()
                .find(|a| a.arg_name == expected.fn_args[i].arg_name);
            assert!(actual_arg.is_some());
            let actual_arg = actual_arg.unwrap();
            assert_eq!(actual_arg.arg_name, expected.fn_args[i].arg_name);
            assert_eq!(actual_arg.arg_type, expected.fn_args[i].arg_type);
        }
    }

    fn verify_display_impl(actual: &DisplayImplDef, expected: DisplayImplDef) {
        assert_eq!(actual.error_name, expected.error_name);
    }

    fn verify_error_impl(actual: &ErrorImplDef, expected: ErrorImplDef) {
        assert_eq!(actual.error_name, expected.error_name);
    }

    fn verify_from_kind_impl(actual: &FromKindImplDef, expected: FromKindImplDef) {
        assert_eq!(actual.error_name, expected.error_name);
        assert_eq!(actual.error_kind_name, expected.error_kind_name);
    }
}
