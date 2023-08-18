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

Freyja relies on `cargo-make` for builds, so you will also need to install that:

```shell
cargo install --force cargo-make
```

1. Clone this repository with `git clone`

### Build

Freyja supports the use of custom library implementations for many of the interfaces with external components. The build depends on a set of environment variables to specify which libraries to use for these implementations. To set these environment variables and build the workspace, run the following command from the repo root:

```shell
cargo make --env-file=./tools/freyja-build.env build
```

With the default values in `tools/freyja-build.env`, this will use the in-memory mock interfaces. For more information on building Freyja with your own libraries, see [the article on using external libraries](docs/external-libs.md).

Note: if you are using the rust-analyzer extension with Visual Studio code, you may need to restart the application any time you change environment variables to avoid incorrect error highlighting or hints

### Run

```shell
cd target/debug
./dts
```

Note that certain choices for the build variables may require other applications to be started as well. In general, everything other than the in-memory libraries will require some kind of external endpoint to be set up.

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
