# In-Memory Mock Cloud Adapter

The In-Memory Mock Cloud Adapter mocks the behavior of the cloud connector from within the memory of the Freyja application. Data which is sent to this adapter will be printed to stdout. This enables a minimal example scenario when working with Freyja. This library contains an implementation of the `CloudAdapter` trait from the contracts.

## Config

The adapter's config is located at `res/config.json` and will be copied to the build output automatically. This file contains the following properties:

- `cloud_service_name` and `host_connection_string`: these are fake values and serve no functional purpose. They are only logged in the output and changing them does not affect the fundamental behavior of the adapter.
