// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

#[macro_export]
macro_rules! response {
    ($status_code:ident) => {
        (axum::http::StatusCode::$status_code, axum::Json("")).into_response()
    };
    ($status_code:ident, $body:expr) => {
        (axum::http::StatusCode::$status_code, axum::Json($body)).into_response()
    };
}

#[macro_export]
macro_rules! ok {
    () => {
        freyja_common::response!(OK)
    };
    ($body:expr) => {
        freyja_common::response!(OK, $body)
    };
}

#[macro_export]
macro_rules! not_found {
    () => {
        freyja_common::response!(NOT_FOUND)
    };
    ($body:expr) => {
        freyja_common::response!(NOT_FOUND, $body)
    };
}

#[macro_export]
macro_rules! server_error {
    () => {
        freyja_common::response!(INTERNAL_SERVER_ERROR)
    };
    ($body:expr) => {
        freyja_common::response!(INTERNAL_SERVER_ERROR, $body)
    };
}
