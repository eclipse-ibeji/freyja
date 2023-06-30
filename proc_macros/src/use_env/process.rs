// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::env;

use quote::format_ident;
use syn::Ident;

use super::parse::UseEnvArgs;

/// Process data for the use_env! macro
///
/// # Arguments
///
/// - `args`: the input arguments
pub(crate) fn process(args: UseEnvArgs) -> UseEnvOutput {
    let use_stmts = args
        .args
        .into_iter()
        .map(|a| UseStmt {
            pieces: vec![get_env_ident(a.env), a.target_type],
        })
        .collect();

    UseEnvOutput { use_stmts }
}

fn get_env_ident(key: Ident) -> Ident {
    match env::var(key.to_string()) {
        Ok(v) => format_ident!("{v}"),
        Err(e) => panic!("Environment variable {key} not found: {e}"),
    }
}

/// An intermediate representation of the use_env output
#[derive(Debug)]
pub(crate) struct UseEnvOutput {
    /// The generated use statments
    pub use_stmts: Vec<UseStmt>,
}

/// An intermediate representation of a use statement
#[derive(Debug)]
pub(crate) struct UseStmt {
    /// The pieces of the use statement which should be separated by "::"
    pub pieces: Vec<Ident>,
}

#[cfg(test)]
mod use_env_process_tests {
    use std::panic::catch_unwind;

    use crate::use_env::parse::UseEnvArg;

    use super::*;

    #[test]
    fn can_lookup_env() {
        let use_env_test = "USE_ENV_TEST";
        let env_val = "VALUE";
        env::set_var(use_env_test, env_val);

        let target_type = "TargetType";

        let input = UseEnvArgs {
            args: vec![UseEnvArg {
                env: format_ident!("{}", use_env_test),
                target_type: format_ident!("{}", target_type),
            }],
        };

        let output = process(input);

        env::remove_var(use_env_test);

        assert!(output.use_stmts.len() == 1);
        assert!(output.use_stmts[0].pieces.len() == 2);
        assert!(output.use_stmts[0].pieces[0] == env_val);
        assert!(output.use_stmts[0].pieces[1] == target_type);
    }

    #[test]
    fn should_panic_when_env_not_found() {
        let use_env_test = "FAKE_ENV_VAR";
        let target_type = "TargetType";

        let input = UseEnvArgs {
            args: vec![UseEnvArg {
                env: format_ident!("{}", use_env_test),
                target_type: format_ident!("{}", target_type),
            }],
        };

        let result = catch_unwind(|| process(input));
        assert!(result.is_err());
    }
}
