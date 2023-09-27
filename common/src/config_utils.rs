// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{path::Path, env};

use config::{ConfigError, File};
use home::home_dir;
use serde::Deserialize;

const CONFIG_DIR: &str = "config";
const DOT_FREYJA_DIR: &str = ".freyja";
const FREYJA_HOME: &str = "FREYJA_HOME";

/// Read config from layered configuration files.
/// 
/// # Arguments
/// - `default_config_path`: The path to the default configuration
/// - `overrides_file_name`: The name of the file(s) that contain overrides
/// - `io_error_handler`: The error handler for IO errors
/// - `deserialize_error_handler`: The error handler for deserialize errors
pub fn read_from_files<TConfig, TError, TPath, TIoErrorHandler, TDeserializeErrorHandler>(
    default_config_path: TPath,
    overrides_file_name: String,
    io_error_handler: TIoErrorHandler,
    deserialize_error_handler: TDeserializeErrorHandler,
    ) -> Result<TConfig, TError> 
where
    TConfig: for<'a> Deserialize<'a>,
    TPath : AsRef<Path>,
    TIoErrorHandler: Fn(std::io::Error) -> TError,
    TDeserializeErrorHandler: FnOnce(ConfigError) -> TError,
{
    // <current_dir>/{config}.json
    let current_dir_config_path = env::current_dir()
        .map_err(&io_error_handler)?
        .join(overrides_file_name.clone());

    let freyja_dir_config_path = match env::var(FREYJA_HOME) {
        Ok(freyja_home) => {
            // $FREYJA_HOME/config/{config}.json
            Path::new(&freyja_home)
                .join(CONFIG_DIR)
                .join(overrides_file_name)
        },
        Err(_) => {
            // $HOME/.freyja/config/mapping_client_config.json
            home_dir()
                .ok_or(io_error_handler(std::io::Error::new(std::io::ErrorKind::Other, "Could not retrieve home directory")))?
                .join(DOT_FREYJA_DIR)
                .join(CONFIG_DIR)
                .join(overrides_file_name)
        }
    };

    let config_store = config::Config::builder()
        .add_source(File::from(default_config_path.as_ref()))
        .add_source(File::from(current_dir_config_path).required(false))
        .add_source(File::from(freyja_dir_config_path).required(false))
        .build()
        .unwrap();

    config_store.try_deserialize().map_err(deserialize_error_handler)
}