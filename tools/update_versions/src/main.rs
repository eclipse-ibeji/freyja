// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{env, fs};

use env_logger::Target;
use glob::glob;

use freyja_common::cmd_utils::parse_args;
use log::{error, info, LevelFilter};
use toml_edit::{value, DocumentMut};

fn main() {
    let args = parse_args(env::args()).expect("Failed to parse args");

    env_logger::Builder::new()
        .filter(None, LevelFilter::Info)
        .target(Target::Stdout)
        .init();

    let dir = match args.get("dir") {
        Some(Some(d)) => d,
        Some(None) => panic!("--dir argument missing value"),
        None => ".",
    };

    let version = match args.get("version") {
        Some(Some(v)) => v,
        _ => panic!("Missing required argument: --version"),
    };

    let dry_run = args.contains_key("dry-run");

    for file in glob(&format!("{}/**/Cargo.toml", dir)).expect("Failed to read glob pattern") {
        match file {
            Err(e) => error!("Unable to read file: {e}"),
            Ok(path) => {
                info!("Updating {:?}...", path.display());
                if !dry_run {
                    let contents = match fs::read_to_string(&path) {
                        Ok(s) => s,
                        Err(e) => {
                            error!("\tError reading file: {e}");
                            continue;
                        }
                    };

                    let mut toml = match contents.parse::<DocumentMut>() {
                        Ok(d) => d,
                        Err(e) => {
                            error!("\tError parsing file: {e}");
                            continue;
                        }
                    };

                    toml["package"]["version"] = value(version);

                    match fs::write(&path, toml.to_string()) {
                        Ok(_) => info!("\tUpdate successful!"),
                        Err(e) => error!("\tError writing file: {e}"),
                    }
                }
            },
        }
    }
}