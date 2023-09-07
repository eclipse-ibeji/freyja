# Writing Custom Adapters and Integrating with Freyja

Freyja allows users to bring their own implementations of various traits which interface with external components. This is achieved by exposing the core functionality of Freyja as a library function and requiring users to author the actual executable package which links everything together. In most cases this can be simplified by using a provided macro, but for scenarios that require more complex setup for the adapters the library function can be called manually after completing setup.

## How to Author a Custom Adapter

Freyja supports custom implementations of the `DigitalTwinAdapter`, `CloudAdapter`, and `MappingClient` interfaces. To refer to these traits in your implementation, you will need to take a dependency on the `freyja-contracts` crate. The following `Cargo.toml` snippet shows how you can include this dependency:

```toml
[dependencies]
freyja-contracts = { git = "https://github.com/eclipse-ibeji/freyja", rev = "<commit hash>" }
```

## How to Author a Freyja Application

To avoid the difficulty that comes with trying to statically link unknown external dependencies via Cargo, Freyja relies on users to implement the actual main executable. To do this, you will need to author a new Cargo package with a binary target (e.g., `cargo new --bin my-app`). This package should take as dependencies any crates that contain your adapter implementations or functionality needed for custom setup steps. In addition, you will need to take dependencies on the `freyja` and `tokio` crates, including the `macros` feature of the `tokio` crate. The following `Cargo.toml` snippet shows how you can include these dependencies:

```toml
[dependencies]
freyja = { git = "https://github.com/eclipse-ibeji/freyja", rev = "<commit hash>" }
tokio = { version = "1.0", features = ["macros"] }
```

A complete sample of the Cargo file for a Freyja application can be found in the [template crate](../examples/template_app/Cargo.toml).

In most cases, the `main.rs` file can be implemented using the `freyja_main!` macro which will take care of writing some boilerplate code for you. This macro only needs adapter typenames as input and will generate the main function signature and body. For an example of how to use this macro, see the code for the [in-memory example](../freyja/examples/in-memory.rs) or the [template crate](../examples/template_app/src/main.rs).

If you have a more complex scenario that requires some additional setup before running the `freyja_main` function, you can instead invoke it manually without using the macro. For an example of how to use this function and how to manually author the main method, see the code for the [in-memory-with-fn example](../freyja/examples/in-memory-with-fn.rs).
