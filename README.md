# Project Freyja

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

Freyja supports the use of custom library implementations for many of the interfaces with external components. For each of these interfaces, an implementation is chosen to statically link at compilation time. This is accomplished through a procedural macro which generates use statements based on enviromnent variables. In order to build, Freyja requires the following environment variables to be set:

Variable|Description|Example
-|-|-
FREYJA_MAPPING_CLIENT|The name of a crate containing the `MappingClient` implementation to use|`in_memory_mock_mapping_client`
FREYJA_DT_ADAPTER|The name of a crate containing the `DigitalTwinAdapter` implementation to use|`in_memory_mock_digital_twin_adapter`
FREYJA_CLOUD_ADAPTER|The name of the crate containing the `CloudAdapter` implementation to use|`in_memory_mock_cloud_adapter`

To quickly set these variables, you can edit the `tools/env-config.sh` file with the desired values. The crates referenced above should re-export their implementation of a trait to `<trait>Impl` so that the Freyja application can find it (for example, add `pub use crate::sample_mapping_client::SampleMappingClient as MappingClientImpl;` to your `lib.rs`). The `dts/Cargo.toml` file includes all of the implementations of these traits as dependencies for convenience, but only the ones selected with this mechanism will get packaged into the final executable.

To set the environment variables and build the workspace, run the following commands from the repo root:

```shell
source tools/env-config.sh
cargo build
```

Note: if you are using the rust-analyzer extension with Visual Studio code, you may need to restart the application any time you change environment variables to avoid incorrect error highlighting or hints

### Run

```shell
cd target/debug
./dts
```

Note that certain choices for the build variables may require other applications to be started as well. In general, everything other than the in-memory libraries will require some kind of external endpoint to be set up.

## Why "Freyja"?

The project takes it name from the Old Norse goddess Freyja. This name was chosen because she is a twin (her brother's name is Freyr) and is associated with *seiðr*, which is magic for seeing and influencing the future. She is also commonly associated with fertility and gold, and rules over the heavenly field of Fólkvangr. She has a necklace called Brísingamen, a cloak of falcon feathers, a boar named Hildisvíni, and rides a chariot pulled by two cats.

## Trademarks

This project may contain trademarks or logos for projects, products, or services. Authorized use of Microsoft
trademarks or logos is subject to and must follow
[Microsoft's Trademark & Brand Guidelines](https://www.microsoft.com/en-us/legal/intellectualproperty/trademarks/usage/general).
Use of Microsoft trademarks or logos in modified versions of this project must not cause confusion or imply Microsoft sponsorship.
Any use of third-party trademarks or logos are subject to those third-party's policies.
