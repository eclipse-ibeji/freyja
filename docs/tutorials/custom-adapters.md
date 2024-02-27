# Writing Custom Adapters and Integrating with Freyja

Freyja allows users to bring their own implementations of various traits which interface with external components. This is achieved by exposing the core functionality of Freyja as a library function and requiring users to author the final binary package to link everything together.

## How to Author a Custom Adapter

Freyja supports custom implementations of the `DigitalTwinAdapter`, `CloudAdapter`, `MappingAdapter`, `DataAdapter`, `DataAdapterFactory`, and `ServiceDiscoveryAdapter` interfaces. To refer to these traits in your implementation, you will need to take a dependency on the `freyja-common` crate. The following `Cargo.toml` snippet shows how you can include this dependency:

```toml
[dependencies]
freyja-common = { git = "https://github.com/eclipse-ibeji/freyja" }
```

For more information about the adapter interfaces, see [the design doc](./../design/README.md#external-interfaces).

## How to Author a Freyja Application

To avoid the difficulty that comes with trying to statically link unknown external dependencies via Cargo, Freyja relies on users to implement the actual main binary package. To do this, you will need to author a new Cargo package with a binary target (e.g., `cargo new --bin my-app`). This package should take dependencies on any crates that contain your adapter implementations or functionality needed for custom setup steps. In addition, you will need to take dependencies on the `freyja` and `tokio` crates, including the `macros` feature of the `tokio` crate. The following `Cargo.toml` snippet shows how you can include these dependencies:

```toml
[dependencies]
freyja = { git = "https://github.com/eclipse-ibeji/freyja" }
tokio = { version = "1.0", features = ["macros"] }
```

In most cases the `main.rs` file can be implemented using the `freyja_main!` macro, which will take care of writing some boilerplate code for you. This macro only needs adapter type names as input and will generate the main function signature and body. For an example of how to use this macro, see the code for the [Standard Freyja Runtime](../../freyja/src/main.rs).

If you have a more complex scenario that requires some additional setup before running the `freyja_main` function, you can instead invoke it manually without using the macro. For an example of how to use this function and how to manually author the main function, see the code for the [in-memory-with-fn example](../../freyja/examples/in-memory-with-fn.rs).

For more examples of Freyja adapters and applications, see the [Ibeji Example Applications repository](https://github.com/eclipse-ibeji/ibeji-example-applications/tree/main/freyja_apps).

## Appendix A

This appendix lists the adapters that are provided in this repository. These can be used as samples for writing your own adapters, and can be mixed and matched with your custom adapters.

### Digital Twin Adapters

- [In-Memory Mock Digital Twin Adapter](../../adapters/digital_twin/in_memory_mock_digital_twin_adapter/README.md): Emulates a Digital Twin Service entirely within the memory of the Freyja application.
- [gRPC Digital Twin Adapter](../../adapters/digital_twin/grpc_digital_twin_adapter/README.md): Communicates with a digital twin service that implements the [Ibeji In-Vehicle Digital Twin Service API](https://github.com/eclipse-ibeji/ibeji/blob/main/interfaces/invehicle_digital_twin/v1/invehicle_digital_twin.proto). This is a "standard adapter" that is suitable for use in production scenarios.

### Mapping Adapters

- [In-Memory Mock Mapping Adapter](../../adapters/mapping/in_memory_mock_mapping_adapter/README.md): Emulates a mapping service entirely within the memory of the Freyja application.
- [gRPC Mapping Adapter](../../adapters/mapping/grpc_mapping_adapter/README.md): Communicates with a mapping service that implements the [Mapping Service API](../../interfaces/mapping_service/v1/mapping_service.proto). This is a "standard adapter" that is suitable for use in production scenarios.

### Cloud Adapters

- [In-Memory Mock Cloud Adapter](../../adapters/cloud/in_memory_mock_cloud_adapter/README.md): Emulates a Cloud Connector entirely within the memory of the Freyja application. Data emitted to this adapter will be printed to the console window.
- [gRPC Cloud Adapter](../../adapters/cloud/grpc_cloud_adapter/README.md): Communicates with a cloud connector that implements the [Cloud Connector API](../../interfaces/cloud_connector/v1/cloud_connector.proto). This is a "standard adapter" that is suitable for use in production scenarios.

### Data Adapters

- [In-Memory Mock Data Adapter](../../adapters/data/in_memory_mock_data_adapter/README.md): Interfaces with the In-Memory Mock Digital Twin Adapter and intended for use with it.
- [Sample gRPC Data Adapter](../../adapters/data/sample_grpc_data_adapter/README.md): Interfaces with providers that communicate via gRPC. Integrated with specific Ibeji samples and the Mock Digital Twin.
- [MQTT Data Adapter](../../adapters/data/mqtt_data_adapter/README.md): Interfaces with providers that communicate via MQTT.
- [Managed Subscribe Data Adapter](../../adapters/data/managed_subscribe_data_adapter/README.md): Interfaces with providers that leverage the managed subscribe feature of Ibeji. This adapter typically requires the MQTT Data Adapter.

### Service Discovery Adapters

- [File Service Discovery Adapter](../../adapters/service_discovery/file_service_discovery_adapter/README.md): Uses a static config file to define service URIs. This is a "standard adapter" that is suitable for use in production scenarios.
- [gRPC Service Discovery Adapter](../../adapters/service_discovery/grpc_service_discovery_adapter/README.md): Communicates with a service discovery system that implements the [Chariott Service Registry API](https://github.com/eclipse-chariott/chariott/blob/main/service_discovery/proto/core/v1/service_registry.proto). This is a "standard adapter" that is suitable for use in production scenarios.
