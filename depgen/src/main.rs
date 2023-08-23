// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{env, fmt::Debug, fs, path::{Path, PathBuf}, process::Command, str};

use toml_edit::Document;

const RESOURCE_DIR_NAME: &str = "res";
const GENERATED_DIR_NAME: &str = "__generated";
const SRC_DIR_NAME: &str = "src";
const CARGO_TEMPLATE_NAME: &str = "Cargo.template.toml";
const CARGO_FILE_NAME: &str = "Cargo.toml";
const LIBRS_FILE_NAME: &str = "lib.rs";

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

    let project_dir = get_current_project_directory(false);
    let res_dir = Path::new(&project_dir)
        .join(RESOURCE_DIR_NAME);
    let generated_dir = Path::new(&project_dir)
        .join(GENERATED_DIR_NAME);
    let generated_src_dir = Path::new(&generated_dir)
        .join(SRC_DIR_NAME);

    println!("Using '{}' as the generated directory", project_dir.display());

    let cargo_template_file = Path::new(&res_dir)
        .join(CARGO_TEMPLATE_NAME);
    let cargo_file = Path::new(&generated_dir)
        .join(CARGO_FILE_NAME);

    let lib_file = Path::new(&generated_src_dir)
        .join(LIBRS_FILE_NAME);

    fs::create_dir_all(generated_src_dir)
        .map_err(|e| format!("Failed to create directory for generated code: {e}"))?;
    write_cargo_toml(&cargo_template_file, &cargo_file)?;
    write_lib(&lib_file)?;
    Ok(())
}

/// Generates and writes the Cargo.toml file for Freyja's dependencies
/// Reads a template that already has package info and overwrites the `dependencies` section
///
/// # Arguments
/// - `path`: The path to the Cargo.toml file to edit
fn write_cargo_toml<P1, P2>(template: &P1, path: &P2) -> Result<(), String>
where
    P1: AsRef<Path> + Debug,
    P2: AsRef<Path> + Debug,
{
    println!("Generating Cargo.toml...");

    let mut toml = fs::read_to_string(template)
        .map_err(|e| format!("Unable to read file at {path:?}: {e:?}"))?
        .parse::<Document>()
        .map_err(|e| format!("Unable to parse Cargo.toml template file: {e:?}"))?;

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

/// Runs cargo locate-project to find the current project
fn get_current_project_directory(workspace: bool) -> PathBuf {
    let mut args = vec!["locate-project", "--message-format=plain"];

    if workspace {
        args.push("--workspace");
    }

    let output = Command::new(env!("CARGO"))
        .args(args)
        .output()
        .unwrap()
        .stdout;

    // This path includes the Cargo.toml filename which needs to get removed
    let cargo_path = Path::new(str::from_utf8(&output).unwrap().trim());
    cargo_path.parent().unwrap().to_path_buf()
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