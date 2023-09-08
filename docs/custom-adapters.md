# Writing Custom Adapters and Integrating with Freyja

Freyja allows users to bring their own implementations of various traits which interface with external components. This is achieved by exposing the core functionality of Freyja as a library function and requiring users to author the final binary package to link everything together. In most cases this can be simplified by using a provided macro, but for scenarios that require more complex setup for the adapters the library function can be called manually.

## How to Author a Custom Adapter

Freyja supports custom implementations of the `DigitalTwinAdapter`, `CloudAdapter`, and `MappingClient` interfaces. To refer to these traits in your implementation, you will need to take a dependency on the `freyja-contracts` crate. The following `Cargo.toml` snippet shows how you can include this dependency:

```toml
[dependencies]
freyja-contracts = { git = "https://github.com/eclipse-ibeji/freyja", rev = "<commit hash>" }
```

Freyja currently requires implementations of the following adapters:

### `DigitalTwinAdapter`

The digital twin adapter interfaces with a digital twin service to get entity information. The [Ibeji Project](https://github.com/eclipse-ibeji/ibeji) is an example of such a service. This interface requires the following function implementations:

- `create_new`: this function will be called by the `freyja_main` function to create an instance of your adapter
- `find_by_id`: queries the digital twin service for information about the requested entity. This information will later be used to set up clients and/or listeners to communicate with that entity's provider.

### `CloudAdapter`

The cloud adapter interfaces with the cloud or a cloud connector to emit data to a digital twin. It's recommended to route communication through a cloud connector on the device to help manage authentication, batching, and other policies that may be useful for automotive scenarios. This interface requires the following function implementations:

- `create_new`: this function will be called by the `freyja_main` function to create an instance of your adapter
- `send_to_cloud`: sends data to the cloud or cloud connector. The request includes a `cloud_signal` property which is a hash map of custom key-value arguments, and the signal value will be converted to a string.

### `MappingClient`

The mapping client interfaces with a mapping service to get information about what signals should be emitted, how those signals need to be transformed and packaged for the cloud, and how often that data should be emitted. This interface requires the following function implementations:

- `create_new`: this function will be called by the `freyja_main` function to create an instance of your adapter
- `check_for_work`: because mappings returned from the `get_mapping` API can be potentially very large, this method is used to first poll for changes before calling that API. If the result is false, then the cartographer will not invoke the `get_mapping` API until it polls again.
- `send_inventory`: this API is currently unused. It is reserved for potential future use, but may also be removed. A default empty implementation is provided for convenience so that this function may be omitted from your trait implementation. It is also safe to use the `unimplemented!()` macro since this function will not be called.
- `get_mapping`: returns mapping information that will be used by Freyja's emitter

## How to Author a Freyja Application

To avoid the difficulty that comes with trying to statically link unknown external dependencies via Cargo, Freyja relies on users to implement the actual main binary package. To do this, you will need to author a new Cargo package with a binary target (e.g., `cargo new --bin my-app`). This package should take dependencies on any crates that contain your adapter implementations or functionality needed for custom setup steps. In addition, you will need to take dependencies on the `freyja` and `tokio` crates, including the `macros` feature of the `tokio` crate. The following `Cargo.toml` snippet shows how you can include these dependencies:

```toml
[dependencies]
freyja = { git = "https://github.com/eclipse-ibeji/freyja", rev = "<commit hash>" }
tokio = { version = "1.0", features = ["macros"] }
```

In most cases, the `main.rs` file can be implemented using the `freyja_main!` macro which will take care of writing some boilerplate code for you. This macro only needs adapter typenames as input and will generate the main function signature and body. For an example of how to use this macro, see the code for the [in-memory example](../freyja/examples/in-memory.rs) or the [mock example](../freyja/examples/mocks.rs).

If you have a more complex scenario that requires some additional setup before running the `freyja_main` function, you can instead invoke it manually without using the macro. For an example of how to use this function and how to manually author the main function, see the code for the [in-memory-with-fn example](../freyja/examples/in-memory-with-fn.rs).
