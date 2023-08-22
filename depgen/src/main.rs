// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{env, fmt::Debug, fs, path::Path};

use toml_edit::Document;

const WORKSPACE_DIR_ENV_VAR: &str = "CARGO_MAKE_WORKSPACE_WORKING_DIRECTORY";
const GENERATED_DIR_RELATIVE_TO_WORKSPACE: &str = "depgen/__generated";
const CARGO_FILE_PATH: &str = "Cargo.toml";
const LIBRS_FILE_PATH: &str = "src/lib.rs";

const DT_ADAPTER_PKG_NAME_ENV_VAR: &str = "FREYJA_DT_ADAPTER_PKG_NAME";
const DT_ADAPTER_PKG_CONFIG_ENV_VAR: &str = "FREYJA_DT_ADAPTER_PKG_CONFIG";
const DT_ADAPTER_STRUCT_ENV_VAR: &str = "FREYJA_DT_ADAPTER_STRUCT";

const CLOUD_ADAPTER_PKG_NAME_ENV_VAR: &str = "FREYJA_CLOUD_ADAPTER_PKG_NAME";
const CLOUD_ADAPTER_PKG_CONFIG_ENV_VAR: &str = "FREYJA_CLOUD_ADAPTER_PKG_CONFIG";
const CLOUD_ADAPTER_STRUCT_ENV_VAR: &str = "FREYJA_CLOUD_ADAPTER_STRUCT";

const MAPPING_CLIENT_PKG_NAME_ENV_VAR: &str = "FREYJA_MAPPING_CLIENT_PKG_NAME";
const MAPPING_CLIENT_PKG_CONFIG_ENV_VAR: &str = "FREYJA_MAPPING_CLIENT_PKG_CONFIG";
const MAPPING_CLIENT_STRUCT_ENV_VAR: &str = "FREYJA_MAPPING_CLIENT_STRUCT";

/// Executes the freyja-depgen application.
/// This generates a crate for use with the Freyja application that bundles dependencies based on environment variables.
fn main() -> Result<(), String> {
    println!("Freyja dependency generator");

    let workspace = get_env(WORKSPACE_DIR_ENV_VAR)?;

    println!("Using workspace directory: {workspace}");
    let cargo_file = Path::new(&workspace)
        .join(GENERATED_DIR_RELATIVE_TO_WORKSPACE)
        .join(CARGO_FILE_PATH);

    let lib_file = Path::new(&workspace)
        .join(GENERATED_DIR_RELATIVE_TO_WORKSPACE)
        .join(LIBRS_FILE_PATH);

    write_cargo_toml(&cargo_file)?;
    write_lib(&lib_file)?;
    Ok(())
}

/// Generates and writes the Cargo.toml file for Freyja's dependencies
/// Reads a template that already has package info and overwrites the `dependencies` section
///
/// # Arguments
/// - `path`: The path to the Cargo.toml file to edit
fn write_cargo_toml<P>(path: &P) -> Result<(), String>
where
    P: AsRef<Path> + Debug,
{
    println!("Generating Cargo.toml...");

    let mut toml = fs::read_to_string(path)
        .map_err(|e| format!("Unable to read file at {path:?}: {e:?}"))?
        .parse::<Document>()
        .map_err(|e| format!("Unable to parse Cargo.toml file: {e:?}"))?;

    let mut dependencies = toml_edit::table();

    // Digital Twin Adapter
    let dt_adapter_package_name = get_env(DT_ADAPTER_PKG_NAME_ENV_VAR)?;
    let dt_adapter_package_config = get_env(DT_ADAPTER_PKG_CONFIG_ENV_VAR)?;
    dependencies[dt_adapter_package_name] = dt_adapter_package_config
        .parse::<toml_edit::Item>()
        .unwrap();

    // Cloud Adapter
    let cloud_adapter_package_name = get_env(CLOUD_ADAPTER_PKG_NAME_ENV_VAR)?;
    let cloud_adapter_package_config = get_env(CLOUD_ADAPTER_PKG_CONFIG_ENV_VAR)?;
    dependencies[cloud_adapter_package_name] = cloud_adapter_package_config
        .parse::<toml_edit::Item>()
        .unwrap();

    // Mapping Client
    let mapping_client_package_name = get_env(MAPPING_CLIENT_PKG_NAME_ENV_VAR)?;
    let mapping_client_package_config = get_env(MAPPING_CLIENT_PKG_CONFIG_ENV_VAR)?;
    dependencies[mapping_client_package_name] = mapping_client_package_config
        .parse::<toml_edit::Item>()
        .unwrap();

    toml["dependencies"] = dependencies;

    fs::write(path, toml.to_string()).map_err(|e| format!("Unable to write file: {e}"))?;

    Ok(())
}

/// Generates and writes the lib.rs file for Freyja's dependencies
///
/// # Arguments
/// - `path`: The path to the lib.rs file
fn write_lib<P>(path: &P) -> Result<(), String>
where
    P: AsRef<Path> + Debug,
{
    println!("Generating lb.rs...");

    // Digital Twin Adapter
    let dt_adapter_use = format!(
        "pub use {} as DigitalTwinAdapterImpl;",
        get_env(DT_ADAPTER_STRUCT_ENV_VAR)?
    );

    // Cloud Adapter
    let cloud_adapter_use = format!(
        "pub use {} as CloudAdapterImpl;",
        get_env(CLOUD_ADAPTER_STRUCT_ENV_VAR)?
    );

    // Mapping Client
    let mapping_client_use = format!(
        "pub use {} as MappingClientImpl;",
        get_env(MAPPING_CLIENT_STRUCT_ENV_VAR)?
    );

    // Sort the use statements and add a newline at the end of the file so that the formatter is happy
    let mut stmts = vec![dt_adapter_use, cloud_adapter_use, mapping_client_use];
    stmts.sort();
    let mut contents = stmts.join("\n");
    contents += "\n";

    fs::write(path, contents).map_err(|e| format!("Unable to write file: {e}"))?;

    Ok(())
}

/// Gets an enironment variable and maps errors to a stringified version of the error.
///
/// # Arguments
/// - `key`: The environment variable to try to read
fn get_env(key: &str) -> Result<String, String> {
    env::var(key).map_err(|e| {
        format!("Unable to get environment variable {key}; did you run this with cargo make? {e:?}")
    })
}
