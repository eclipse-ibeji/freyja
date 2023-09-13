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
/// - `default_config`: The default configuration setting
pub fn read_from_files<TPath1, TPath2, TConfig, TError, TIoErrorHandler, TDeserializeErrorHandler>(
    default_config_path: TPath1,
    override_config_name: TPath2,
    io_error_handler: TIoErrorHandler,
    deserialize_error_handler: TDeserializeErrorHandler,
    ) -> Result<TConfig, TError> 
where
    TPath1 : AsRef<Path>,
    TPath2 : AsRef<Path> + Clone,
    TConfig: for<'a> Deserialize<'a>,
    TIoErrorHandler: Fn(std::io::Error) -> TError,
    TDeserializeErrorHandler: FnOnce(ConfigError) -> TError,
{
    // <current_dir>/{config}.json
    let current_dir_config_path = env::current_dir()
        .map_err(&io_error_handler)?
        .join(override_config_name.clone());

    let freyja_dir_config_path = match env::var(FREYJA_HOME) {
        Ok(freyja_home) => {
            // $FREYJA_HOME/config/{config}.json
            Path::new(&freyja_home)
                .join(CONFIG_DIR)
                .join(override_config_name)
        },
        Err(_) => {
            // $HOME/.freyja/config/mapping_client_config.json
            home_dir()
                .ok_or(io_error_handler(std::io::Error::new(std::io::ErrorKind::Other, "Could not retrieve home directory")))?
                .join(DOT_FREYJA_DIR)
                .join(CONFIG_DIR)
                .join(override_config_name)
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