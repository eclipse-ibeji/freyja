// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{env, fs};

use toml_edit::Document;

const GENERATED_DIRECTORY_RELATIVE_TO_WORKSPACE: &str = "depgen/__generated";
const CARGO_FILE_PATH: &str = "Cargo.toml";
const LIBRS_FILE_PATH: &str = "src/lib.rs";

fn main() -> Result<(), String> {
    println!("Freyja dependency generator");

    let workspace = env::var("CARGO_MAKE_WORKSPACE_WORKING_DIRECTORY")
        .map_err(|_| "Unable to get workspace directory; did you run this with cargo make?"
        .to_string())?;

    println!("Workspace directory: {workspace}");
    let cargo_file = format!("{}/{}/{}",
        workspace.clone(),
        GENERATED_DIRECTORY_RELATIVE_TO_WORKSPACE,
        CARGO_FILE_PATH);

    let lib_file = format!("{}/{}/{}",
        workspace,
        GENERATED_DIRECTORY_RELATIVE_TO_WORKSPACE,
        LIBRS_FILE_PATH);

    write_cargo_toml(&cargo_file)?;
    write_lib(&lib_file)?;
    Ok(())
}

fn write_cargo_toml(path: &String) -> Result<(), String> {
    println!("Generating Cargo.toml...");

    let mut toml = fs::read_to_string(path)
        .map_err(|e| format!("Unable to read file at {path}: {e:?}"))?
        .parse::<Document>()
        .map_err(|e| format!("Unable to parse Cargo.toml file: {e:?}"))?;

    let mut dependencies = toml_edit::table();
    dependencies["in-memory-mock-digital-twin-adapter"] = "{ path = \"../../digital_twin_adapters/in_memory_mock_digital_twin_adapter\" }".parse::<toml_edit::Item>().unwrap();

    toml["dependencies"] = dependencies;

    fs::write(path, toml.to_string())
        .map_err(|e| format!("Unable to write file: {e}"))?;

    Ok(())
}

fn write_lib(path: &String) -> Result<(), String> {
    println!("Generating lb.rs...");

    let contents = "pub use in_memory_mock_digital_twin_adapter::in_memory_mock_digital_twin_adapter::InMemoryMockDigitalTwinAdapter as DigitalTwinAdapterImpl;";
    fs::write(path, contents)
        .map_err(|e| format!("Unable to write file: {e}"))?;

    Ok(())
}