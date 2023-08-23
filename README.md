# Project Freyja

<a href="https://github.com/eclipse-ibeji/freyja/actions/workflows/rust-ci.yml"><img alt="build: N/A" src="https://img.shields.io/github/actions/workflow/status/eclipse-ibeji/freyja/rust-ci.yml"></a>
<a href="https://github.com/eclipse-ibeji/freyja/issues"><img alt="issues: N/A" src="https://img.shields.io/github/issues/eclipse-ibeji/freyja"></a>
<img src="https://img.shields.io/badge/status-maintained-green.svg" alt="status: maintained">
<a href="https://github.com/eclipse-ibeji/freyja/blob/main/LICENSE"><img alt="license: MIT" src="https://img.shields.io/github/license/eclipse-ibeji/freyja"></a>

- [Introduction](#introduction)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Build](#build)
  - [Run](#run)
- [Why "Freyja"?](#why-freyja)
- [Trademarks](#trademarks)

## Introduction

Project Freyja is an ESDV application which enables synchronization between the digital twin state on the vehicle (the "instance digital twin") and the digital twin state in the cloud (the "canonical digital twin").

For more information on Freyja's design and how it works, see [the design document](docs/design/README.md).

## Getting Started

### Prerequisites

This guide uses `apt` as the package manager in the examples. You may need to substitute your own package manager in place of `apt` when going through these steps.

1. Install git and rust:

```shell
sudo apt update
sudo apt install -y git snapd
sudo snap install rustup --classic
```

The rust toolchain version is managed by the `rust-toolchain.toml` file, so once you install `rustup` there is no need to manually install a toolchain or set a default.

1. Clone this repository with `git clone`

### Build

Freyja supports the use of custom library implementations for many of the interfaces with external components. To accomplish this, there is a code generation step that is a required prerequisite for the build.

#### Pre-Build

1. Set up your environment. The instructions below are the recommended way to do this, but any other method of setting environment variables will also work if you prefer not to edit your personal cargo config file(s).
    1. Copy the template file from `<repo-root>/depgen/res/config.template.toml` to your personal cargo config at `$CARGO_HOME/cargo.toml` (defaults to `$HOME/.cargo/cargo.toml` on Linux if `$CARGO_HOME` is not set). If you don't already have a cargo config file you can copy the entire contents of the template, otherwise just copy the contents of the `[env]` section into your own config file's `[env]` table.
    1. Modify the values to configure the dependencies you want to use. By default, the template specifies the in-memory mocks. For more information on how to configure your environment see the [documentation on using external libraries](docs/external-libs.md).
1. Starting from the repo root, build and run the dependency generator application:

```shell
pushd depgen
cargo run
popd
```

This step will need to be executed again any time you change the environment variables in your cargo config to use different dependencies.

#### Build

Once the pre-build steps are completed, you can build the Freyja application with cargo:

```shell
cargo build
```

### Run

You can run the Freyja application using cargo:

```shell
cargo run --bin dts
```

Note that the dependencies chosen during the pre-build steps may require other applications to be started as well. In general, everything other than the in-memory libraries will require some kind of external endpoint to be set up.

<!--alex disable he-she her-him brothers-sisters-->
## Why "Freyja"?

The project takes it name from the Old Norse goddess Freyja. This name was chosen because she is a twin (her brother's name is Freyr) and is associated with *seiðr*, which is magic for seeing and influencing the future. She is also commonly associated with fertility and gold, and rules over the heavenly field of Fólkvangr. She has a necklace called Brísingamen, a cloak of falcon feathers, a boar named Hildisvíni, and rides a chariot pulled by two cats.
<!--alex enable he-she her-him brothers-sisters-->

## Trademarks

This project may contain trademarks or logos for projects, products, or services. Authorized use of Microsoft
trademarks or logos is subject to and must follow
[Microsoft's Trademark & Brand Guidelines](https://www.microsoft.com/en-us/legal/intellectualproperty/trademarks/usage/general).
Use of Microsoft trademarks or logos in modified versions of this project must not cause confusion or imply Microsoft sponsorship.
Any use of third-party trademarks or logos are subject to those third-party's policies.
