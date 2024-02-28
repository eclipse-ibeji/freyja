// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{
    io::{stderr, Write},
    path::PathBuf,
};

use uuid::Uuid;

/// A test fixture which helps manage tests with gRPC adapters
pub struct GRPCTestFixture {
    /// The path to a file in the temp directory.
    /// Note that this fixture will not create anything at this location,
    /// it will just generate a filename and clean up the file during teardown.
    pub socket_path: PathBuf,
}

impl GRPCTestFixture {
    /// Create a new `GRPCTestFixture` that generates a unique `socket_path`
    pub fn new() -> Self {
        Self {
            socket_path: std::env::temp_dir()
                .as_path()
                .join(Uuid::new_v4().as_hyphenated().to_string()),
        }
    }
}

impl Default for GRPCTestFixture {
    /// Create a new `GRPCTestFixture` that generates a unique `socket_path`
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for GRPCTestFixture {
    /// Cleans up the fixture by deleting the file at the socket path
    fn drop(&mut self) {
        match std::fs::remove_file(&self.socket_path) {
            Ok(_) => {}
            Err(e) => {
                write!(stderr(), "Error cleaning up `GRPCTestFixture`: {e:?}")
                    .expect("Error writing to stderr");
            }
        }
    }
}
