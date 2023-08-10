// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::future::Future;

use log::debug;
use tokio::time::{sleep, Duration};

/// Retry an async function that returns any error.
///
/// # Arguments
/// * `max_retries` - The maximum number of retries.
/// * `duration_between_retries` - The duration between retries.
/// * `function` - The function to retry.
pub async fn retry_async_function<T, E, Fut, F: FnMut() -> Fut>(
    max_retries: u32,
    duration_between_retries: Duration,
    mut function: F,
) -> Result<T, E>
where
    Fut: Future<Output = Result<T, E>>,
{
    let mut last_error: Result<T, E>;
    let mut retries = 0;

    loop {
        match function().await {
            Ok(t) => return Ok(t),
            Err(error) => {
                last_error = Err(error);
            }
        }
        debug!("Retrying function call.");
        sleep(duration_between_retries).await;

        retries += 1;

        if retries == max_retries {
            break;
        }
    }
    last_error
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::rc::Rc;

    async fn test_function(attempts: Rc<RefCell<u32>>) -> Result<(), ()> {
        let mut attempts = attempts.borrow_mut();
        *attempts += 1;
        if *attempts == 3 {
            Ok(())
        } else {
            Err(())
        }
    }

    #[tokio::test]
    async fn test_retry_async_function() {
        const MAX_RETRIES: u32 = 3;

        let attempts = Rc::new(RefCell::new(0));
        let mut result = retry_async_function(MAX_RETRIES, Duration::from_secs(1), || {
            test_function(attempts.clone())
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(*attempts.borrow(), MAX_RETRIES);

        *attempts.borrow_mut() = 4;
        result = retry_async_function(MAX_RETRIES, Duration::from_secs(1), || {
            test_function(attempts.clone())
        })
        .await;
        assert!(result.is_err());
    }
}
