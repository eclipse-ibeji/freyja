# Using External Libraries with Freyja

Freyja allows users to bring their own implementations of various traits which interface with external components. This is accomplished by generating a crate at build time which bundles these dependencies and re-exports the relevant structs under aliases that Freyja can interact with. This allows users to customize the behavior of Freyja by changing the build time environment instead of having to modify source code!

## How to Author an External Dependency

## Building Freyja with External Dependencies

1. Copy and modify the `tools/freyja-build.env` file as necessary
1. Build the Freyja application:

```shell
cargo make --env-file=/path/to/your/env/file build
```

Note that it is not necessary to build your dependencies separately, as they will be built automatically as required.

## Freyja Dependency Generator

The Freyja dependency generator helps with bringing in custom libraries and linking then with the main Freyja application. The dependency generator is an executable located in the `depgen` folder, and the generated files are placed in `depgen/__generated`. This folder contains some placeholder files so that cargo can check the workspace properly. These placeholders will be part of the repo when you clone it, but changes will be ignored. Initially the placeholders will have references to the in-memory mock libraries so the workspace will actually build without running the dependency generator. However, it is strongly recommended to use `cargo make` to interact with the project (e.g. `cargo make build`) so that the dependency generator runs automatically and picks up your build configuration since all of the `cargo make` commands for this project will invoke the generator.

The dependency generator will generate a small crate which bundles Freyja's external dependencies. This includes a `lib.rs` file which contains re-exports of relevant structs and a `Cargo.toml` file to package the dependencies. The dependency generator requires the following environment variables to be defined:

Variable Name|Description
-|-
`FREYJA_DT_ADAPTER_PKG_NAME`|The name of the package to use for the cloud adapter. This should match the name defined in the target package's Cargo.toml.
`FREYJA_DT_ADAPTER_PKG_CONFIG`|The package config for the digital twin adapter. This string should be a valid toml item and will be used as the value for the package name key in the dependencies table.
`FREYJA_DT_ADAPTER_STRUCT`|The struct to use as the implementation of the digital twin adapter. This value will be used as part of a use statement in the generated `lib.rs` and should include the crate name.
`FREYJA_CLOUD_ADAPTER_PKG_NAME`|The name of the package to use for the cloud adapter. This should match the name defined in the target package's Cargo.toml.
`FREYJA_CLOUD_ADAPTER_PKG_CONFIG`|The package config for the cloud adapter. This string should be a valid toml item and will be used as the value for the package name key in the dependencies table.
`FREYJA_CLOUD_ADAPTER_STRUCT`|The struct to use as the implementation of the cloud adapter. This value will be used as part of a use statement in the generated `lib.rs` and should include the crate name.
`FREYJA_MAPPING_CLIENT_PKG_NAME`|The name of the package to use for the mapping client. This should match the name defined in the target package's Cargo.toml.
`FREYJA_MAPPING_CLIENT_PKG_CONFIG`|The package config for the mapping client. This string should be a valid toml item and will be used as the value for the package name key in the dependencies table.
`FREYJA_MAPPING_CLIENT_STRUCT`|The struct to use as the implementation of the mapping client. This value will be used as part of a use statement in the generated `lib.rs` and should include the crate name.

For samples of these values, see `tools/freyja-build.env`, which is configured to use the in-memory mocks by default. These environment variables are passed to `cargo make` with the `--env-file` argument. To configure your own environment, copy the `tools/freyja-build.env` file, modify the values as appropriate, and use it for your `cargo make` commands.

The dependency generator is configured as a dependency for all of the `cargo make` tasks in this project, though it can also be run manually if necessary. Since the dependency generator relies on the environment set up by `cargo make` it must be executed with the `cargo make --env-file=/path/to/your/env/file freyja-depgen` command.