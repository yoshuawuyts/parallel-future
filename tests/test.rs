use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

#[cfg(feature = "async-std")]
use async_std::task::sleep;
#[cfg(feature = "tokio")]
use tokio::time::sleep;

use futures_concurrency::prelude::*;
use parallel_future::prelude::*;

#[cfg_attr(feature = "async-std", async_std::test)]
#[cfg_attr(feature = "tokio", tokio::test)]
async fn spawn() {
    let res = async { "nori is a horse" }.par().await;
    assert_eq!(res, "nori is a horse");
}

#[cfg_attr(feature = "async-std", async_std::test)]
#[cfg_attr(feature = "tokio", tokio::test)]
async fn with_join() {
    // A parallel execution of two futures
    let a = async { 1 }.par();
    let b = async { 2 }.par();

    let res: [i32; 2] = (a, b).join().await.into();
    assert_eq!(res.iter().sum::<i32>(), 3);
}

#[cfg_attr(feature = "async-std", async_std::test)]
#[cfg_attr(feature = "tokio", tokio::test)]
async fn is_lazy() {
    let polled = Arc::new(Mutex::new(false));
    let polled_2 = polled.clone();
    let _res = async move {
        *polled_2.lock().unwrap() = true;
    }
    .par();

    sleep(Duration::from_millis(500)).await;
    assert_eq!(*polled.lock().unwrap(), false);
}
