# Using External Libraries with Freyja

Freyja allows users to bring their own implementations of various traits which interface with external components. This is accomplished by generating a crate at build time which bundles these dependencies and re-exports the relevant structs under aliases that Freyja can interact with. This allows users to customize the behavior of Freyja by changing the build time environment instead of having to modify source code!

## Freyja Dependency Generator

The Freyja dependency generator helps with bringing in custom libraries and linking then with the main Freyja application. The dependency generator is an executable located in the `/depgen` folder, and the generated files are placed in `depgen/__generated`. This folder contains some placeholder files so that cargo can check the workspace properly, but the workspace will not acutally build until the dependency generator is run. It is strongly recommended to use `cargo make` to interact with the project (e.g. `cargo make build`) so that the dependency generator runs automatically since all of the `cargo make` commands for this project will invoke the generator.

The dependnecy generator will generate a small crate which bundles Freyja's external dependencies. This includes a `lib.rs` file which contains re-exports of relevant structs and a `Cargo.toml` file to package the dependencies. The dependency generator requires the following environment variables to be defined:

### TODO

These environment variables can be configured in a file such as `tools/freyja-build-env` and passed to `cargo make` with the `--env-file` argument. To configure your own environment, copy the `tools/freyja-build.env` file, modify the values as appropriate, and use it for your `cargo make` commands.