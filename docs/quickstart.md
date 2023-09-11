<!-- language-all: shell -->
# Freyja Quickstart Guide

The Freyja project provides some example adapter implementations that can be used to get started quickly and experiment with Freyja without needing to write any code. For more information about the example adapters that Freyja provides, including links to documentation on how to configure them for more complex scenarios, see the [Appendix](#appendix-a).

## Build

In order to run the examples mentioned in this document, you will first need to build them with the following command:

```shell
cargo build --examples
```

## Example Scenarios

### In-Memory Example

This example uses the in-memory mock adapters to emulate the behavior of external components from entirely within the memory of the Freyja application. This example does not require any other services to be configured or running in order to function properly, and there will be no external API calls made by the Freyja core components.

This example is ideal for getting started with minimal effort or configuration. However, it does not offer precise control over how the mocked interfaces behave during runtime. This example is most commonly used for testing scenarios.

To run this sample, run the following command:

```shell
cargo run --example in-memory
```

Note that there is also an `in-memory-with-fn` example with identical behavior. The difference between these two examples is that they show different ways of integrating the same adapters with the Freyja core components, which is an advanced topic covered in the [Custom Adapters Guide](./custom-adapters.md).

### Mock Services Example

This example uses the Mock Digital Twin Service and Mock Mapping Service. The behavior is very similar to the in-memory example, but with two key differences:

1. The mapping client and digital twin adapter function as clients to external services rather than handling everything in-memory. These external services are mock versions of the mapping and digital twin services which run as binaries on the same device.
1. The mock services allow for more precise control over when their state changes. Users can advance the state of the applications by interacting with their terminal interfaces.

This example is ideal if you need to manually control when signals or mappings are added or removed from the application, thus affecting what data gets emitted by Freyja. This example is most commonly used for demo scenarios.

To run this sample, follow these steps:

1. Run the Mock Digital Twin Service. To do so, open a new terminal window and run the following:

       cargo run --bin mock-digital-twin

    Note that with the default configuration, the mock is initialized with no entities activated. Whenever you press <kbd>Enter</kbd> in the mock's terminal window, the mock's state will change to include additional entites that will be returned by the `find_by_id` API. Using the default configuration, up to three entities can be added one at a time when pressing <kbd>Enter</kbd>.

1. Run the Mock Mapping Service. To do so, open a new terminal window and run the following:

        cargo run --bin mock-mapping-service

    Note that with the default configuration, the mock is initialized with no mappings activated. Whenever you press <kbd>Enter</kbd> in the mock's terminal window, the mock's state will change to include additional mappings that will be returned by the `get_mapping` API. Using the default configuration, up to three mappings can be added one at a time when pressing <kbd>Enter</kbd>.

1. Run the example. To do so, run the following in the original terminal window:

        cargo run --example mocks

# Appendix A

This appendix lists the sample adapters that are provided in this repository. These are self-contained and do not require anything outside of this repository to be set up. More detailed adapters which interface with external components can be found in the [Ibeji Example Applications repository](https://github.com/eclipse-ibeji/ibeji-example-applications).

## Digital Twin Adapters

- [In-Memory Mock Digital Twin Adapter](../digital_twin_adapters/in_memory_mock_digital_twin_adapter/README.md): Emulates a Digital Twin Service entirely within the memory of the Freyja application.
- [Mock Digital Twin Adapter](../digital_twin_adapters/mock_digital_twin_adapter/README.md): Communicates with the [Mock Digital Twin](../mocks/mock_digital_twin/README.md), which is an executable that mocks the Digital Twin Service. The behavior is very similar to the in-memory mock, but the application is interactive and allows users to add or remove entities from the mocked digital twin by pressing enter to advance through configurable states.

## Mapping Clients

- [In-Memory Mock Mapping Client](../mapping_clients/in_memory_mock_mapping_client/README.md): Emulates a mapping service entirely within the memory of the Freyja application.
- [Mock Mapping Service Client](../mapping_clients/mock_mapping_service_client/README.md): Communicates with the [Mock Mapping Service](../mocks/mock_mapping_service/README.md), which is an executable that mocks a Mapping Service. The behavior is very similar to the in-memory mock, but the application is interactive and allows users to add or remove mappings by pressing enter to advance through configurable states.

## Digital Twin Adapters

- [In-Memory Mock Cloud Adapter](../cloud_adapters/in_memory_mock_cloud_adapter/README.md): Emulates a Cloud Connector entirely within the memory of the Freyja application. When data is emitted to this adapter it will be printed to the console window.
