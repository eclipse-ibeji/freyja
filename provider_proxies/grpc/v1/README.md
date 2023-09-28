# GRPC Provider Proxy

The GRPC Provider Proxy interfaces with providers which support GRPC. It acts as a consumer for digital twin providers. This proxy supports the operations `Get` and `Subscribe` as defined for the [Ibeji mixed sample](https://github.com/eclipse-ibeji/ibeji/tree/main/samples/mixed). To use this proxy with other providers, those providers will need to support the same APIs as the provider in the Ibeji mixed sample.

## Configuration

This proxy's default config is located at `res/grpc_proxy_config.default.json` and will be copied to the build output automatically. This proxy supports the following configuration settings:

- `consumer_address`: The listen address for the proxy's consumer

You can override the default values by defining your own `grpc_proxy_config.json`. The adapter will probe for and unify config in this order, with values near the end of the list taking higher precedence:

- The default config
- A `grpc_proxy_config.json` file in the working directory of the executable (for example, the directory you were in when you ran the `cargo run` command)
- `$FREYJA_HOME/config/grpc_proxy_config.json`. If you have not set a `$FREYJA_HOME` directory, this defaults to:
  - Unix: `$HOME/.freyja/config/grpc_proxy_config.json`
  - Windows: `%USERPROFILE%\.freyja\config\grpc_proxy_config.json` (note that Windows support is not guaranteed by Freyja or this adapter)