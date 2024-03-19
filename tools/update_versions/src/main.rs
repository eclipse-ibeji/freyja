// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{env, fs};

use glob::glob;

use freyja_common::cmd_utils::parse_args;
use toml_edit::{value, DocumentMut};

fn main() {
    let args = parse_args(env::args()).expect("Failed to parse args");

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
            Err(e) => println!("Unable to read file: {e}"),
            Ok(path) => {
                println!("Checking {:?}...", path.display());

                let contents = match fs::read_to_string(&path) {
                    Ok(s) => s,
                    Err(e) => {
                        println!("\tError reading file: {e}");
                        continue;
                    }
                };

                let mut toml = match contents.parse::<DocumentMut>() {
                    Ok(d) => d,
                    Err(e) => {
                        println!("\tError parsing file: {e}");
                        continue;
                    }
                };

                // This check prevents the app from updating workspaces, which don't have a version configured
                if toml.contains_table("package")
                    && toml["package"].as_table().unwrap().contains_key("version")
                    && toml["package"]["version"].is_str()
                {
                    let current_version = &toml["package"]["version"].as_str().unwrap();

                    if dry_run {
                        println!("\tWould update version: {current_version} -> {version}");
                    } else {
                        toml["package"]["version"] = value(version);

                        match fs::write(&path, toml.to_string()) {
                            Ok(_) => println!("\tUpdate successful!"),
                            Err(e) => println!("\tError writing file: {e}"),
                        }
                    }
                } else {
                    println!("\tNo package version to update");
                }
            }
        }

        // Helps with readability
        println!();
    }
}
