#!/bin/bash

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.
# SPDX-License-Identifier: MIT

set -e
cd "$(dirname "$0")/.."

sudo apt-get install -y plantuml

plantuml -tsvg .