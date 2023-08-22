// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{env, fmt::Debug, fs, path::Path};

use toml_edit::Document;

const GENERATED_DIR_RELATIVE_TO_WORKSPACE: &str = "depgen/__generated";
const CARGO_FILE_PATH: &str = "Cargo.toml";
const LIBRS_FILE_PATH: &str = "src/lib.rs";

fn main() -> Result<(), String> {
    println!("Freyja dependency generator");

    let workspace = get_env("CARGO_MAKE_WORKSPACE_WORKING_DIRECTORY")?;

    println!("Workspace directory: {workspace}");
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
    let dt_adapter_package_name = get_env("FREYJA_DT_ADAPTER_PKG_NAME")?;
    let dt_adapter_package_config = get_env("FREYJA_DT_ADAPTER_PKG_CONFIG")?;
    dependencies[dt_adapter_package_name] = dt_adapter_package_config
        .parse::<toml_edit::Item>()
        .unwrap();

    // Cloud Adapter
    let cloud_adapter_package_name = get_env("FREYJA_CLOUD_ADAPTER_PKG_NAME")?;
    let cloud_adapter_package_config = get_env("FREYJA_CLOUD_ADAPTER_PKG_CONFIG")?;
    dependencies[cloud_adapter_package_name] = cloud_adapter_package_config
        .parse::<toml_edit::Item>()
        .unwrap();

    // Mapping Client
    let mapping_client_package_name = get_env("FREYJA_MAPPING_CLIENT_PKG_NAME")?;
    let mapping_client_package_config = get_env("FREYJA_MAPPING_CLIENT_PKG_CONFIG")?;
    dependencies[mapping_client_package_name] = mapping_client_package_config
        .parse::<toml_edit::Item>()
        .unwrap();

    toml["dependencies"] = dependencies;

    fs::write(path, toml.to_string()).map_err(|e| format!("Unable to write file: {e}"))?;

    Ok(())
}

fn write_lib<P>(path: &P) -> Result<(), String>
where
    P: AsRef<Path> + Debug,
{
    println!("Generating lb.rs...");

    // Digital Twin Adapter
    let dt_adapter_use = format!(
        "pub use {} as DigitalTwinAdapterImpl;",
        get_env("FREYJA_DT_ADAPTER_STRUCT")?
    );

    // Cloud Adapter
    let cloud_adapter_use = format!(
        "pub use {} as CloudAdapterImpl;",
        get_env("FREYJA_CLOUD_ADAPTER_STRUCT")?
    );

    // Mapping Client
    let mapping_client_use = format!(
        "pub use {} as MappingClientImpl;",
        get_env("FREYJA_MAPPING_CLIENT_STRUCT")?
    );

    // Sort the use statements and add a newline at the end of the file so that the formatter is happy
    let mut stmts = vec![dt_adapter_use, cloud_adapter_use, mapping_client_use];
    stmts.sort();
    let mut contents = stmts.join("\n");
    contents += "\n";

    fs::write(path, contents).map_err(|e| format!("Unable to write file: {e}"))?;

    Ok(())
}

fn get_env(key: &str) -> Result<String, String> {
    env::var(key).map_err(|e| {
        format!("Unable to get environment variable {key}; did you run this with cargo make? {e:?}")
    })
}
