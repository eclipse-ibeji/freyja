# Config Overrides

Many Freyja components support configuration overrides to enable users to override default settings and provide custom configuration. This is achieved with configuration layering. Components using this configuration strategy will define a default config, which is often suitable for basic scenarios or getting started quickly. This default config can be overridden at runtime using custom values. When loading configuration, a component will probe for and unify config in the following order, with values near the end of the list taking higher precedence:

- The default config
- A config file in the working directory of the executable (for example, the directory you were in when you ran the `cargo run` command)
- `$FREYJA_HOME/config/{config_name}.json`. If you have not set a `$FREYJA_HOME` directory, this defaults to:
  - Unix: `$HOME/.freyja/config/{config_name}.json`
  - Windows: `%USERPROFILE%\.freyja\config\{config_name}.json` (note that Windows support is not guaranteed by Freyja)

Because the config is layered, the overrides can be partially defined and only specify the top-level configuration fields that should be overridden. Anything not specified in an override file will use the default value.
