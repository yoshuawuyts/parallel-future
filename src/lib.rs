//! fluent async task experiments
//! 
//! Read more about it in the [post on postfix
//! spawn](https://blog.yoshuawuyts.com/postfix-spawn/). This is an experiment
//! in moving the design of tasks from a model where "tasks are async threads"
//! to a model where"tasks are parallel futures".
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
#![feature(into_future)]

use pin_project::{pin_project, pinned_drop};
use std::future::{Future, IntoFuture};
use std::marker::PhantomData;
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
pub struct JoinHandle<Fut: Future, K: sealed::Kind> {
    builder: Option<Builder<Fut, K>>,
    #[pin]
    handle: Option<task::JoinHandle<Fut::Output>>,
}

impl<Fut: Future, K: sealed::Kind> JoinHandle<Fut, K> {
    /// Detaches the task to let it keep running in the background.
    pub fn detach(self) {
        std::mem::forget(self);
    }
}

impl<Fut: Future + 'static> Future for JoinHandle<Fut, Local> {
    type Output = <Fut as Future>::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        if let Some(builder) = this.builder.take() {
            this.handle
                .replace(builder.builder.local(builder.future).unwrap());
        }
        Pin::new(&mut this.handle.as_pin_mut().unwrap()).poll(cx)
    }
}

impl<Fut> Future for JoinHandle<Fut, NonLocal>
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
impl<Fut: Future, K: sealed::Kind> PinnedDrop for JoinHandle<Fut, K> {
    fn drop(self: Pin<&mut Self>) {
        let mut this = self.project();
        let handle = this.handle.take().unwrap();
        let _ = handle.cancel();
    }
}

/// Extend the `Future` trait.
pub trait FutureExt: Future + Sized {
    /// Spawn a task on a thread pool
    fn spawn(self) -> Builder<Self, NonLocal>
    where
        Self: Send,
    {
        Builder {
            kind: PhantomData,
            future: self,
            builder: async_std::task::Builder::new(),
        }
    }

    /// Spawn a task on the same thread.
    fn spawn_local(self) -> Builder<Self, Local> {
        Builder {
            kind: PhantomData,
            future: self,
            builder: async_std::task::Builder::new(),
        }
    }
}

impl<F> FutureExt for F where F: Future {}

/// Sealed trait to determine what type of bulider we got.
mod sealed {
    pub trait Kind {}
}

/// A local builder.
#[derive(Debug)]
pub struct Local;
impl sealed::Kind for Local {}

/// A nonlocal builder.
#[derive(Debug)]
pub struct NonLocal;
impl sealed::Kind for NonLocal {}

/// Task builder that configures the settings of a new task.
#[derive(Debug)]
#[must_use = "async builders do nothing unless you call `into_future` or `.await` them"]
pub struct Builder<Fut: Future, K: sealed::Kind> {
    kind: PhantomData<K>,
    future: Fut,
    builder: async_std::task::Builder,
}

impl<Fut: Future, K: sealed::Kind> Builder<Fut, K> {
    /// Set the name of the task.
    pub fn name(mut self, name: String) -> Builder<Fut, K> {
        self.builder = self.builder.name(name);
        self
    }
}

impl<Fut> IntoFuture for Builder<Fut, NonLocal>
where
    Fut::Output: Send,
    Fut: Future + Send + 'static,
{
    type Output = Fut::Output;

    type IntoFuture = JoinHandle<Fut, NonLocal>;

    fn into_future(self) -> Self::IntoFuture {
        JoinHandle {
            builder: Some(self),
            handle: None,
        }
    }
}

impl<Fut> IntoFuture for Builder<Fut, Local>
where
    Fut: Future + 'static,
{
    type Output = Fut::Output;

    type IntoFuture = JoinHandle<Fut, Local>;

    fn into_future(self) -> Self::IntoFuture {
        JoinHandle {
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
    fn spawn_local() {
        async_std::task::block_on(async {
            let res = async { "nori is a horse" }.spawn_local().await;
            assert_eq!(res, "nori is a horse");
        })
    }

    #[test]
    fn name() {
        async_std::task::block_on(async {
            let res = async { "nori is a horse" }
                .spawn_local()
                .name("meow".into())
                .await;
            assert_eq!(res, "nori is a horse");
        })
    }
}
