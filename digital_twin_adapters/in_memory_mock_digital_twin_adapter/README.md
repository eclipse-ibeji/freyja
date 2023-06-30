# In Memory Mock Digital Twin Adapter

The In Memory Mock Digital Twin Adapter mocks the behavior of the in-vehicle digital twin service (e.g. Ibeji) from within the memory of the Freyja application. This enables the simplest demo possible when working with the application. This library contains an implementation of the `DigitalTwinAdapter` trait from the contracts.

## Configuration

The adapter supports three methods of configuration:

### Default Config File

This is the default method of configuration and is the one that will be used in most cases. The default adapter can be created by calling the `InMemoryMockDigitalTwinAdapter::create_new()` method and inspecting the returned `Result`. The mock config is located at `res/config.json` and it will be copied to the build output automatically. This file is a list of `ProviderConfig` objects with the following properties:

- `dtdl`: the provider's dtdl that contains the following information:
  - `url`: the url that can be used to "call" this provider. This is a stand-in for whatever the provider contact info is from Ibeji. This is used as the key when calling `subscribe` and `get`.
  - `id`: this is used as the key when calling `find_by_id`.
  - `description` and `name`: these are currently unused by the Freyja application and are included for human readability and/or potential future use.
  - `operations`: operations that the provider supports, such as `Get`, `Subscribe`, and `Invoke`.
- `values`: configuration for how the mocks provider's values should change. There are two options:
  - `Static`: a value that doesn't change.
  - `Stepwise`: a value which increases or decreases at a set interval between starting and ending values. Acts like a static value upon reaching the end of the range.

For non-`Static` value configurations, the value returned for a certain signal will be a function of the number of times the method has been previously called for that signal. For example, when getting the value of a `Stepwise`-configured signal before reaching `end`, the value returned will be `start + delta * n`, where `n` is the number of times the method has been called before.

### Custom Config File

The InMemoryMockDigitalTwinAdapter provides support for a custom config file should the default config be insufficient. An InMemoryMockDigitalTwinAdapter with custom config is created with the `InMemoryMockDigitalTwinAdapter::from_config_file(config_path)` method.

### In-Memory Configuration

The client also includes support for config generated in-memory rather than read from a file. This is primarily used in test cases to avoid dealing with files. A client with generated config is created with the `InMemoryMockDigitalTwinAdapter::from_config(config)` method.
