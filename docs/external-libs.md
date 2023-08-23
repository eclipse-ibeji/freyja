# Using External Libraries with Freyja

Freyja allows users to bring their own implementations of various traits which interface with external components. This is accomplished by generating a crate as a pre-build step which bundles these dependencies and re-exports the relevant structs under aliases that Freyja can interact with. This allows users to customize the behavior of Freyja by changing the build time environment instead of having to modify source code!

## How to Author an External Dependency

Freyja supports custom implementations of the `DigitalTwinAdapter`, `CloudAdapter`, and `MappingClient` interfaces. To write your own, make a new library crate, add `dts-contracts` as a dependency, and implement the interface. Note the package name and the way you can refer to the package in a toml file (e.g., `{ path = "some/path" }` or `{ git = "some.git/url" }`), as well as the fully qualified name of the struct that implements one of the mentioned structs. You will need these to set up the environment for the dependency generator (see [Environment Variables](#environment-variables)).

It is not necessary to build your dependencies separately. Since they will be included in the generated `Cargo.toml` file, Cargo will take care of locating and building your dependencies when you build the Freyja application.

Note that each trait implementation must live in a separate crate. You cannot put all of the implementations in one crate.

## Freyja Dependency Generator

The Freyja dependency generator helps with bringing in custom libraries and linking them with the main Freyja application. The dependency generator is an executable located in the `depgen` folder, and the generated files are placed in `depgen/__generated`.

**The Freyja application will not build successfully until you run the dependency generator at least once!** This is because the `dts/Cargo.toml` file refers to a crate that does not exist until the generator creates it. You don't need to re-run the generator unless you change the environment config mentioned below, so generally you only need to run it once and can build and rebuild Freyja normally with Cargo afterwards.

The dependency generator will generate a small crate which bundles Freyja's external dependencies. This includes a `lib.rs` file which contains re-exports of relevant structs and a `Cargo.toml` file to package the dependencies. The generated package is excluded from the source tree so that users can freely customize their build without having git track their individual changes.

### Environment Variables

The dependency generator requires the following environment variables to be defined:

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

For samples of these values, see `depgen/res/config.template.toml`, which is configured to use the in-memory mocks by default. It's recommended to copy the contents of this file into a personal config file (such as `$CARGO_HOME/config.toml` or `/.cargo/config.toml`), but any other way of setting the environment variables will also work.

### Build and Run

To build and run the dependency generator, you can use `cargo build` and/or `cargo run` from the `<repo-root>/depgen` folder.
