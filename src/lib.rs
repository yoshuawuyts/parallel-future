//! Fluent async task experiments
//!
//! Read more about it in the ["postfix
//! spawn" post](https://blog.yoshuawuyts.com/postfix-spawn/). This is an experiment
//! moving tasks from a model where "tasks are async threads" to a model where:
//! "tasks are parallel futures".
//!
//! This means tasks will no longer start unless explicitly `.await`ed, dangling
//! tasks become a thing of the past, and by default async Rust will act
//! structurally concurrent.
//!
//! # Examples
//!
//! ```
//! use tasky::prelude::*;
//!
//! async_std::task::block_on(async {
//!     let res = async { "nori is a horse" }
//!         .spawn()
//!         .name("meow".into())
//!         .await;
//!     assert_eq!(res, "nori is a horse");
//! })
//! ```

#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, unreachable_pub)]

use pin_project::{pin_project, pinned_drop};
use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::task::{Context, Poll};

use async_std::task;

/// The `tasky` prelude.
pub mod prelude {
    pub use super::FutureExt as _;
}

/// A handle representing a task.
#[derive(Debug)]
#[pin_project(PinnedDrop)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct ParallelFuture<Fut: Future> {
    builder: Option<Builder<Fut>>,
    #[pin]
    handle: Option<task::JoinHandle<Fut::Output>>,
}

impl<Fut> Future for ParallelFuture<Fut>
where
    Fut: Future + Send + 'static,
    Fut::Output: Send + 'static,
{
    type Output = <Fut as Future>::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        if let Some(builder) = this.builder.take() {
            this.handle
                .replace(builder.builder.spawn(builder.future).unwrap());
        }
        Pin::new(&mut this.handle.as_pin_mut().unwrap()).poll(cx)
    }
}

/// Cancel a task when dropped.
#[pinned_drop]
impl<Fut: Future> PinnedDrop for ParallelFuture<Fut> {
    fn drop(self: Pin<&mut Self>) {
        let mut this = self.project();
        let handle = this.handle.take().unwrap();
        let _ = handle.cancel();
    }
}

/// Extend the `Future` trait.
pub trait FutureExt: Future + Sized {
    /// Spawn a task on a thread pool
    fn spawn(self) -> Builder<Self>
    where
        Self: Send,
    {
        Builder {
            future: self,
            builder: async_std::task::Builder::new(),
        }
    }
}

impl<F> FutureExt for F where F: Future {}

/// Task builder that configures the settings of a new task.
#[derive(Debug)]
#[must_use = "async builders do nothing unless you call `into_future` or `.await` them"]
pub struct Builder<Fut: Future> {
    future: Fut,
    builder: async_std::task::Builder,
}

impl<Fut: Future> Builder<Fut> {
    /// Set the name of the task.
    pub fn name(mut self, name: String) -> Builder<Fut> {
        self.builder = self.builder.name(name);
        self
    }
}

impl<Fut> IntoFuture for Builder<Fut>
where
    Fut::Output: Send,
    Fut: Future + Send + 'static,
{
    type Output = Fut::Output;

    type IntoFuture = ParallelFuture<Fut>;

    fn into_future(self) -> Self::IntoFuture {
        ParallelFuture {
            builder: Some(self),
            handle: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::prelude::*;

    #[test]
    fn spawn() {
        async_std::task::block_on(async {
            let res = async { "nori is a horse" }.spawn().await;
            assert_eq!(res, "nori is a horse");
        })
    }

    #[test]
    fn name() {
        async_std::task::block_on(async {
            let res = async { "nori is a horse" }
                .spawn()
                .name("meow".into())
                .await;
            assert_eq!(res, "nori is a horse");
        })
    }
}
