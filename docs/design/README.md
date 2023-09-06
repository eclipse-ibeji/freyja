# Freyja Design Specification

- [Introduction](#introduction)
- [Architecture](#architecture)
  - [Cartographer](#cartographer)
  - [Emitter](#emitter)
  - [External Interfaces](#external-interfaces)
  - [Mapping Service](#mapping-service)
- [Future Work](#future-work)

## Introduction

The Software-Defined Vehicle will need to connect to the cloud for scenarios such as data synchronization, command processing, and analytics. However, this is a hard problem when different vehicles with different digital twin models need to synchronize on a shared "canonical model" in the cloud. The Freyja project solves this problem by creating a flexible framework for digital twin synchronization in the vehicle.

## Architecture

At its core, Freyja consists of two main components: the **cartographer** and the **emitter**. In addition to these core components, there are multiple interfaces with external components that define how Freyja interacts with the cloud and the rest of the Software Defined Vehicle. There are interfaces for the in-vehicle digital twin (such as Ibeji), the mapping service (provided by customers), and the cloud digital twin provider (such as Azure or AWS). Below is a high-level diagram of the Freyja components:

![Component Diagram](../diagrams/freyja_components.svg)

In a typical life cycle, the Freyja application will start up, discover Ibeji via Chariott or a static configuration, then connect to the mapping service to obtain a digital twin map. This map will define which signals need to be synced with the cloud digital twin, how often they need to be synced, and how the data should be transformed or packaged. Once a mapping is obtained, Freyja will connect to the providers and begin emitting their data according to the mapping. In case of changes on either the device or vehicle side, the mapping is dynamic and can be updated as required to add, remove, or modify the signals that are being emitted.

### Cartographer

The cartographer is the core component responsible for managing the digital twin mapping. The current implementation is very minimal and will poll the mapping client for updates. If there is an update pending, the cartographer will download it and update the application's stored mapping info. This is currently implemented as a shared application state which both the cartographer and emitter have access to.

![Sequence Diagram](../diagrams/mapping_service_to_cartographer_sequence.svg)

### Emitter

The emitter is the core component responsible for actually emitting data. The emitter supports intervals at a per-signal level to enable signals to have different requirements on how often they are synced with the cloud. Note that once a signal is added to the mapping and picked up by the cartographer, it can take up to `min(`*`I`*`)` before the signal is emitted, where *`I`* is the set of intervals for signals already being tracked.

![Digital Twin Sequence Diagram](../diagrams/digital_twin_to_emitter_sequence.svg)

### External Interfaces

Freyja has the following interfaces for external components:

Component|Examples|Interface Trait|Description
-|-|-|-
In-Vehicle Digital Twin|Ibeji and its providers|`DigitalTwinAdapter`|Communicates with the in-vehicle digital twin to get signal values during emission. Often referred to as "DT Adapter"
Mapping Service|`MockMappingService`, other customer-provided implementations|`MappingClient`|Communicates with the mapping service
Cloud Digital Twin|Azure, AWS|`CloudAdapter`|Communicates with the cloud digital twin provider

All of these interfaces are defined as traits with async methods in the `contracts/src` folder. The implementation of each trait is selected via dynamic compilation using Freyja's `use_env!` macro. This repository contains some sample or mock implementations of each trait.

#### In-Vehicle Digital Twin Interface

Freyja communicates with the in-vehicle digital twin to get signal values for emission via the `DigitalTwinAdapter` trait. This trait defines the following functions:

- `create_new`: Generates a `Box<dyn DigitalTwinAdapter>` and serves as the integration point for the core Freyja components
- `find_provider_by_id`: Finds a digital twin provider by id
- `get_signal_value`: Gets the value of a signal from a provider. Note that this API is subject to change or removal since it is a synchronous request but the overall SDV architecture calls for "asynchronous gets" with a callback. The current API is simplified for easier prototyping

Although this component is built with the same pluggable model as other external interfaces, it is being designed closely together with other SDV components. As a result, it is strongly suggested to use the provided SDV implementation of this interface, and this implementation should be sufficient for most production scenarios.

#### Mapping Client Interface

Freyja communicates with a mapping service via the `MappingClient` trait to get information about how to package data during emission. This trait defines the following functions:

- `create_new`: Generates a `Box<dyn MappingClient>` and serves as the integration point for the core Freyja components
- `check_for_work`: Queries the mapping service to check for pending work
- `send_inventory`: Sends the current provider inventory to the mapping service so that it can compute a mapping. Note that this API is subject to change or removal since some details of the mapping client request sequence are still under active design. As a result, this API is currently unused by the cartographer but is reserved for potential future use.
- `get_mapping`: Gets the mapping for this vehicle

For more information about the mapping service and how this interface is used, see the [Mapping Service](#mapping-service) section.

#### Cloud Digital Twin Interface

Freyja communicates with a the cloud digital twin via the `CloudAdapter` trait. This trait defines the following functions:

- `create_new`: Generates a `Box<dyn CloudAdapter>` and serves as the integration point for the core Freyja components
- `send_to_cloud`: Sends a data package to the cloud digital twin

### Mapping Service

Freyja relies on an external mapping service to define how data should be synced to the cloud. The implementation of this service is intentionally left undefined as it's expected that it will vary on a per-customer basis. We only define the interface that the Freyja application expects and provide some sample mock services.

At a high level, this component should be able to identify the vehicle making a request and either look up or compute a mapping for that vehicle. This could be done with a static vehicle-id-to-mapping database, or it might be more dynamic and linked to the cloud digital twin solution to compute mappings on the fly.

The reference architecture here specifies the mapping service as a cloud service with which Freyja communicates, though an alternate reference architecture may have Freyja communicating with another application on the vehicle which caches data from the cloud service. Yet another potential architecture may leverage the vehicle's OTA solution to update the mapping data on a local mapping service rather than having a dedicated cloud mapping service. Freyja supports a flexible pluggable system to enable customers to select the implementation that best meets their needs.

## Future Work

Freyja currently only supports device-to-cloud (D2C) scenarios. Cloud-to-device (C2D) scenarios are planned for the future, though there are no current designs for this feature.

In addition, Freyja currently only supports a single protocol for communication with providers, as well as a single data schema. In reality, providers in the same vehicle may support multiple protocols and schemas.
