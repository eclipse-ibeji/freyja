# Project Freyja

<a href="https://github.com/eclipse-ibeji/freyja/actions/workflows/rust-ci.yml"><img alt="build: N/A" src="https://img.shields.io/github/actions/workflow/status/eclipse-ibeji/freyja/rust-ci.yml"></a>
<a href="https://github.com/eclipse-ibeji/freyja/issues"><img alt="issues: N/A" src="https://img.shields.io/github/issues/eclipse-ibeji/freyja"></a>
<img src="https://img.shields.io/badge/status-maintained-green.svg" alt="status: maintained">
<a href="https://github.com/eclipse-ibeji/freyja/blob/main/LICENSE"><img alt="license: MIT" src="https://img.shields.io/github/license/eclipse-ibeji/freyja"></a>

- [Introduction](#introduction)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Using Freyja](#using-freyja)
- [Why "Freyja"?](#why-freyja)
- [Trademarks](#trademarks)

## Introduction

Project Freyja is an ESDV application which enables synchronization between the digital twin state on the vehicle (the "instance digital twin") and the digital twin state in the cloud (the "canonical digital twin").

For more information on Freyja's design and how it works, see [the design document](docs/design/README.md).

## Getting Started

### Prerequisites

This guide uses `apt` as the package manager in the examples. You may need to substitute your own package manager in place of `apt` when going through these steps.

1. Install required packages. These packages are necessary to clone the repo, install the Rust toolchain, and build Freyja's dependencies:

    ```shell
    sudo apt update
    sudo apt install -y \
        git \
        snapd \
        gcc \
        protobuf-compiler \
        pkg-config libssl-dev \
        cmake
    ```

1. Install the Rust Toolchain:

    ```shell
    sudo snap install rustup --classic
    ```

    The rust toolchain version is managed by the `rust-toolchain.toml` file, so once you install `rustup` there is no need to manually install a toolchain or set a default.

1. Clone this repository with `git clone`

### Using Freyja

Freyja supports a default runtime that is integrated with a set of standard adapters. To build and run the Standard Freyja Runtime, run the following command:

```shell
cargo run -p freyja
```

This runtime will support the following set of standard adapters:

- [gRPC Digital Twin Adapter](adapters/digital_twin/grpc_digital_twin_adapter/README.md) (which supports [Eclipse Ibeji](https://github.com/eclipse-ibeji/ibeji))
- [gRPC Mapping Adapter](adapters/mapping/grpc_mapping_adapter/README.md)
- [gRPC Cloud Adapter](adapters/cloud/grpc_cloud_adapter/README.md)
- [Sample gRPC Data Adapter](adapters/data/sample_grpc_data_adapter/README.md)
- [MQTT Data Adapter](adapters/data/mqtt_data_adapter/README.md)
- [Managed Subscribe Data Adapter](adapters/data/managed_subscribe_data_adapter/README.md) (which supports [Eclipse Agemo](https://github.com/eclipse-chariott/agemo))
- [File Service Discovery Adapter](adapters/service_discovery/file_service_discovery_adapter/README.md)
- [gRPC Service Discovery Adapter](adapters/service_discovery/grpc_service_discovery_adapter/README.md) (which supports [Eclipse Chariott](https://github.com/eclipse-chariott/chariott))

Freyja also supports custom adapter implementations for more specific scenarios. To learn about custom adapters and how to implement and use them, see the [Custom Adapters Guide](docs/tutorials/custom-adapters.md).

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
