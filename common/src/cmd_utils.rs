// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT/

use std::{collections::HashMap, str::FromStr};

use log::LevelFilter;
use proc_macros::error;

/// Gets the command-line arguments.
/// Arguments should be formatted as `--key[=value]`.
/// The output is a `HashMap` containing keys mapped to values.
/// The keys have all leading `-` characters removed.
/// If an argument is passed as a flag without a corresponding value,
/// then an entry is created in the hash map with a value of `None`.
/// Note that this does not support values containing the `=` character,
/// nor does it support multiple instances of the same key!
///
/// # Arguments
/// - `args`: the command-line arguments (usually obtained with `env::args()`)
pub fn parse_args<T>(args: T) -> Result<HashMap<String, Option<String>>, ParseArgsError>
where
    T: Iterator<Item = String>,
{
    let mut result = HashMap::new();

    // First item in this list is the program name
    for arg in args.skip(1) {
        let mut split = arg.split('=');
        let key = match split.next() {
            // Note that unwrapping here will always succeed because `s` is guaranteed to be at least 3 chars long
            Some(s) if s.len() > 2 && s.get(..2) == Some("--") => s.get(2..).unwrap().to_owned(),
            _ => return Err(ParseArgsErrorKind::KeyParseError { arg }.into()),
        };

        // If split.next() returns None here, then this was a flag argument and the call to map also returns None.
        let val = split.next().map(|v| v.to_owned());

        if split.next().is_some() {
            return Err(ParseArgsErrorKind::ValueParseError { arg }.into());
        }

        if result.contains_key(&key) {
            return Err(ParseArgsErrorKind::DuplicateKeys { key }.into());
        }

        result.insert(key, val);
    }

    Ok(result)
}

/// Gets the log level from the provided args.
/// Returns `ParseError` if the argument could not be parsed into a `LevelFilter`,
/// and `MissingValue` if a log level argument was present but did not specify a value.
///
/// # Arguments
/// - `args`: the parsed command line arguments.
/// - `default`: the default value to use if there is no log-level argument.
pub fn get_log_level(
    args: &HashMap<String, Option<String>>,
    default: LevelFilter,
) -> Result<LevelFilter, GetLogLevelError> {
    match args.get("log-level") {
        Some(Some(l)) => LevelFilter::from_str(l)
            .map_err(|_| GetLogLevelErrorKind::ParseError { val: l.to_owned() }.into()),
        Some(None) => Err(GetLogLevelErrorKind::MissingValue.into()),
        None => Ok(default),
    }
}

error! {
    ParseArgsError {
        KeyParseError {
            arg: String
        },
        ValueParseError {
            arg: String
        },
        DuplicateKeys {
            key: String
        },
    }
}

error! {
    GetLogLevelError {
        MissingValue,
        ParseError {
            val: String
        },
    }
}

#[cfg(test)]
mod cmd_utils_tests {
    use super::*;

    #[test]
    fn parse_args_parses_valid_input() {
        let cmd = "cmd".to_owned();
        let key = "foo".to_owned();
        let val = "bar".to_owned();
        let flag = "flag".to_owned();

        let input = vec![
            // Mimics the behavior of real command-line args which have the command as the first entry.
            cmd.clone(),
            format!("--{key}={val}"),
            format!("--{flag}"),
        ];

        let result = parse_args(input.into_iter());

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.get(&key), Some(&Some(val)));
        assert_eq!(result.get(&flag), Some(&None));
        assert!(!result.contains_key("dne"));
        assert!(!result.contains_key(&cmd));
    }

    #[test]
    fn parse_args_returns_error_when_arg_too_short() {
        let cmd = "cmd".to_owned();
        let invalid_arg = "1".to_owned();

        let input = vec![
            // Mimics the behavior of real command-line args which have the command as the first entry.
            cmd.clone(),
            invalid_arg.clone(),
        ];

        let result = parse_args(input.into_iter());

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err.kind() {
            ParseArgsErrorKind::KeyParseError { arg } => assert_eq!(arg, invalid_arg),
            _ => panic!("Expected KeyParseError"),
        }
    }

    #[test]
    fn parse_args_returns_error_when_arg_missing_dashes() {
        let cmd = "cmd".to_owned();
        let invalid_arg = "arg".to_owned();

        let input = vec![
            // Mimics the behavior of real command-line args which have the command as the first entry.
            cmd.clone(),
            invalid_arg.clone(),
        ];

        let result = parse_args(input.into_iter());

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err.kind() {
            ParseArgsErrorKind::KeyParseError { arg } => assert_eq!(arg, invalid_arg),
            _ => panic!("Expected KeyParseError"),
        }
    }

    #[test]
    fn parse_args_returns_error_when_too_many_equals() {
        let cmd = "cmd".to_owned();
        let invalid_arg = "--key=foo=bar".to_owned();

        let input = vec![
            // Mimics the behavior of real command-line args which have the command as the first entry.
            cmd.clone(),
            invalid_arg.clone(),
        ];

        let result = parse_args(input.into_iter());

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err.kind() {
            ParseArgsErrorKind::ValueParseError { arg } => assert_eq!(arg, invalid_arg),
            _ => panic!("Expected ValueParseError"),
        }
    }

    #[test]
    fn parse_args_returns_error_when_duplicate_keys() {
        let cmd = "cmd".to_owned();
        let dup_key = "key".to_owned();

        let input = vec![
            // Mimics the behavior of real command-line args which have the command as the first entry.
            cmd.clone(),
            format!("--{dup_key}"),
            format!("--{dup_key}"),
        ];

        let result = parse_args(input.into_iter());

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err.kind() {
            ParseArgsErrorKind::DuplicateKeys { key } => assert_eq!(key, dup_key),
            _ => panic!("Expected DuplicateKeys"),
        }
    }

    #[test]
    fn get_log_level_returns_value() {
        let mut args = HashMap::new();
        args.insert("log-level".to_owned(), Some("debug".to_owned()));

        let result = get_log_level(&args, LevelFilter::Info);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, LevelFilter::Debug);
    }

    #[test]
    fn get_log_level_returns_parse_error() {
        let mut args = HashMap::new();
        let invalid_value = "foo".to_owned();
        args.insert("log-level".to_owned(), Some(invalid_value.clone()));

        let result = get_log_level(&args, LevelFilter::Info);

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err.kind() {
            GetLogLevelErrorKind::ParseError { val } => assert_eq!(val, invalid_value),
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn get_log_level_returns_missing_value() {
        let mut args = HashMap::new();
        args.insert("log-level".to_owned(), None);

        let result = get_log_level(&args, LevelFilter::Info);

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err.kind() {
            GetLogLevelErrorKind::MissingValue => {}
            _ => panic!("Expected MissingValue"),
        }
    }

    #[test]
    fn get_log_level_returns_default() {
        let args = HashMap::new();
        let default = LevelFilter::Info;

        let result = get_log_level(&args, default);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, default);
    }
}
