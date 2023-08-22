// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod error;
mod use_env;

use proc_macro::TokenStream;

/// Creates use statements based on environment variables
///
/// # Arguments
///
/// - `ts`: A token stream with the following grammatical syntax:
///
/// *UseEnvPredicateList*:
///
/// &nbsp;&nbsp;&nbsp;&nbsp;*UseEnvPredicate* (`,` *UseEnvPredicate*)\*
///
/// *UseEnvPredicate*:
///
/// &nbsp;&nbsp;&nbsp;&nbsp;IDENTIFIER`::`IDENTIFIER
///
/// # Examples
///
/// Assuming that `FREYJA_MAPPING_CLIENT` is an environment variable with the value `mock_mapping_client`
/// and that `FREYJA_DT_ADAPTER` is an environment variable with the value `mock_dt_adapter`,
///
/// `use_env!(FREYJA_MAPPING_CLIENT::MappingClientImpl, FREYJA_DT_ADAPTER::DtAdapterImpl);`
///
/// is equivalent to:
///
/// `use mock_mapping_client::MappingClientImpl;`
///
/// `use mock_dt_adapter::DtAdapterImpl;`
#[proc_macro]
#[deprecated = "deprecated in favor of using freyja-depgen to target dependencies based on environment variables"]
pub fn use_env(ts: TokenStream) -> TokenStream {
    use_env::use_env(ts.into()).into()
}

/////////////////////////////////////////////////
/// error macro
/////////////////////////////////////////////////

/// Generates an error type that is an error and error kind pair, along with methods to construct them.
///
/// # Arguments
///
/// - `ts`: A token stream with the following grammatical syntax:
///
/// *ErrorPredicate*:
///
/// &nbsp;&nbsp;&nbsp;&nbsp;IDENTIFIER `{` *ErrorKindVariant* (`,` *ErrorKindVariant*)\* `}`
///
/// *ErrorKindVariant*:
///
/// &nbsp;&nbsp;&nbsp;&nbsp;IDENTIFIER (*VariantTuple* | *VariantStruct*)?
///
/// *VariantTuple*:
///
/// &nbsp;&nbsp;&nbsp;&nbsp;`(`[*TupleFields*](https://doc.rust-lang.org/reference/items/structs.html)`)`
///
/// *VariantStruct*:
///
/// &nbsp;&nbsp;&nbsp;&nbsp;`{`[*StructFields*](https://doc.rust-lang.org/reference/items/structs.html)`}`
///
/// The types used in variant tuples and structs must implement `Debug`, `PartialEq`, and `Clone`.
///
/// # Examples
///
/// `error!{ ExampleError { Foo, Bar(f32), Baz { b: u8, a: u16, z: u32 } } }`
#[proc_macro]
pub fn error(ts: TokenStream) -> TokenStream {
    error::error(ts.into()).into()
}
