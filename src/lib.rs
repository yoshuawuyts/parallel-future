//! Structured parallel execution for async Rust.
//!
//! > Concurrency is a system-structuring mechanism, parallelism is a resource.
//!
//! This is a replacement for the common `Task` idiom. Rather than providing a
//! separate family of APIs for concurrency and parallelism, this library
//! provides a `ParallelFuture` type. When this type is scheduled concurrently
//! it will provide parallel execution.
//!
//! # Examples
//!
//! ```
//! use parallel_future::prelude::*;
//! use futures_concurrency::prelude::*;
//!
//! async_std::task::block_on(async {
//!     let a = async { 1 }.par();        // ← returns `ParallelFuture`
//!     let b = async { 2 }.par();        // ← returns `ParallelFuture`
//!
//!     let (a, b) = (a, b).join().await; // ← concurrent `.await`
//!     assert_eq!(a + b, 3);
//! })
//! ```
//!
//! # Limitations
//!
//! Rust does not yet provide a mechanism for async destructors. That means that
//! on an early return of any kind, Rust can't guarantee that certain
//! asynchronous operations run before others. This is a language-level
//! limitation with no existing workarounds possible. `ParallelFuture` is designed to
//! work with async destructors once they land.
//!
//! `ParallelFuture` starts lazily and does not provide a manual `detach`
//! method. However it can be manually polled once and then passed to
//! `mem::forget`, which will keep the future running on another thread. In the
//! absence of unforgettable types (linear types), Rust cannot prevent
//! `ParallelFuture`s from becoming unmanaged (dangling).

#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, unreachable_pub)]

use pin_project::{pin_project, pinned_drop};
use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::task::{Context, Poll};

use async_std::task;

/// The `parallel-future` prelude.
pub mod prelude {
    pub use super::IntoFutureExt as _;
}

/// A parallelizable future.
///
/// This type is constructed by the [`par`][crate::IntoFutureExt::par] method on [`IntoFutureExt`][crate::IntoFutureExt].
///
/// # Examples
///
/// ```
/// use parallel_future::prelude::*;
/// use futures_concurrency::prelude::*;
///
/// async_std::task::block_on(async {
///     let a = async { 1 }.par();        // ← returns `ParallelFuture`
///     let b = async { 2 }.par();        // ← returns `ParallelFuture`
///
///     let (a, b) = (a, b).join().await; // ← concurrent `.await`
///     assert_eq!(a + b, 3);
/// })
/// ```
#[derive(Debug)]
#[pin_project(PinnedDrop)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct ParallelFuture<Fut: IntoFuture> {
    into_future: Option<Fut>,
    #[pin]
    handle: Option<task::JoinHandle<Fut::Output>>,
}

impl<Fut> Future for ParallelFuture<Fut>
where
    Fut: IntoFuture,
    Fut::IntoFuture: Send + 'static,
    Fut::Output: Send + 'static,
{
    type Output = <Fut as IntoFuture>::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        if this.handle.is_none() {
            let into_fut = this.into_future.take().unwrap().into_future();
            let handle = task::spawn(into_fut.into_future());
            *this.handle = Some(handle);
        }
        Pin::new(&mut this.handle.as_pin_mut().unwrap()).poll(cx)
    }
}

/// Cancel the `ParallelFuture` when dropped.
#[pinned_drop]
impl<Fut: IntoFuture> PinnedDrop for ParallelFuture<Fut> {
    fn drop(self: Pin<&mut Self>) {
        let mut this = self.project();
        if let Some(handle) = this.handle.take() {
            let _ = handle.cancel();
        }
    }
}

/// Extend the `Future` trait.
pub trait IntoFutureExt: IntoFuture + Sized
where
    <Self as IntoFuture>::IntoFuture: Send + 'static,
    <Self as IntoFuture>::Output: Send + 'static,
{
    /// Convert this future into a parallelizable future.
    ///
    /// # Examples
    ///
    /// ```
    /// use parallel_future::prelude::*;
    /// use futures_concurrency::prelude::*;
    ///
    /// async_std::task::block_on(async {
    ///     let a = async { 1 }.par();        // ← returns `ParallelFuture`
    ///     let b = async { 2 }.par();        // ← returns `ParallelFuture`
    ///
    ///     let (a, b) = (a, b).join().await; // ← concurrent `.await`
    ///     assert_eq!(a + b, 3);
    /// })
    /// ```
    fn par(self) -> ParallelFuture<Self> {
        ParallelFuture {
            into_future: Some(self),
            handle: None,
        }
    }
}

impl<Fut> IntoFutureExt for Fut
where
    Fut: IntoFuture,
    <Fut as IntoFuture>::IntoFuture: Send + 'static,
    <Fut as IntoFuture>::Output: Send + 'static,
{
}

#[cfg(test)]
mod test {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use async_std::task;

    use super::prelude::*;

    #[test]
    fn spawn() {
        async_std::task::block_on(async {
            let res = async { "nori is a horse" }.par().await;
            assert_eq!(res, "nori is a horse");
        })
    }

    #[test]
    fn is_lazy() {
        async_std::task::block_on(async {
            let polled = Arc::new(Mutex::new(false));
            let polled_2 = polled.clone();
            let _res = async move {
                *polled_2.lock().unwrap() = true;
            }
            .par();

            task::sleep(Duration::from_millis(500)).await;
            assert_eq!(*polled.lock().unwrap(), false);
        })
    }
}
