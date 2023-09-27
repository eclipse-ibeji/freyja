// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{env, path::Path};

use config::{ConfigError, File};
use home::home_dir;
use serde::Deserialize;

const CONFIG_DIR: &str = "config";
const DOT_FREYJA_DIR: &str = ".freyja";
const FREYJA_HOME: &str = "FREYJA_HOME";

/// Read config from layered configuration files.
/// Uses `{config_file_name}.default.{config_file_ext}` as the base configuration,
/// then searches for overrides named `{config_file_name}.{config_file_ext}` in the current directory and `$FREYJA_HOME`.
/// If `$FREYJA_HOME` is not set, it defaults to `$HOME/.freyja`.
///
/// # Arguments
/// - `config_file_name`: The config file name without an extension. This is used to construct the file names to search for
/// - `config_file_ext`: The config file extension. This is used to construct the file names to search for
/// - `default_config_path`: The path to the directory containing the default configuration
/// - `io_error_handler`: The error handler for IO errors
/// - `deserialize_error_handler`: The error handler for deserialize errors
pub fn read_from_files<TConfig, TError, TPath, TIoErrorHandler, TDeserializeErrorHandler>(
    config_file_name: &str,
    config_file_ext: &str,
    default_config_path: TPath,
    io_error_handler: TIoErrorHandler,
    deserialize_error_handler: TDeserializeErrorHandler,
) -> Result<TConfig, TError>
where
    TConfig: for<'a> Deserialize<'a>,
    TPath: AsRef<Path>,
    TIoErrorHandler: Fn(std::io::Error) -> TError,
    TDeserializeErrorHandler: FnOnce(ConfigError) -> TError,
{
    let default_config_filename = format!("{}.default.{}", config_file_name, config_file_ext);
    let default_config_file = default_config_path.as_ref().join(default_config_filename);

    let overrides_filename = format!("{}.{}", config_file_name, config_file_ext);

    // <current_dir>/{config}.json
    let current_dir_config_path = env::current_dir()
        .map_err(&io_error_handler)?
        .join(overrides_filename.clone());

    let freyja_dir_config_path = match env::var(FREYJA_HOME) {
        Ok(freyja_home) => {
            // $FREYJA_HOME/config/{config}.json
            Path::new(&freyja_home)
                .join(CONFIG_DIR)
                .join(overrides_filename)
        }
        Err(_) => {
            // $HOME/.freyja/config/mapping_client_config.json
            home_dir()
                .ok_or(io_error_handler(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Could not retrieve home directory",
                )))?
                .join(DOT_FREYJA_DIR)
                .join(CONFIG_DIR)
                .join(overrides_filename)
        }
    };

    let config_store = config::Config::builder()
        .add_source(File::from(default_config_file))
        .add_source(File::from(current_dir_config_path).required(false))
        .add_source(File::from(freyja_dir_config_path).required(false))
        .build()
        .unwrap();

    config_store
        .try_deserialize()
        .map_err(deserialize_error_handler)
}
