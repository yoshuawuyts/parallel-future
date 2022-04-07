//! fluent async task experiments
//!
//! # Examples
//!
//! ```
//! // tbi
//! ```

#![forbid(unsafe_code, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, unreachable_pub)]
#![feature(into_future)]

use std::future::{Future, IntoFuture};
use std::marker::PhantomData;

use async_std::task::JoinHandle;

/// The `tasky` prelude.
pub mod prelude {
    pub use super::FutureExt as _;
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

impl<Fut: Future + 'static> IntoFuture for Builder<Fut, Local> {
    type Output = Fut::Output;

    type IntoFuture = JoinHandle<Fut::Output>;

    fn into_future(self) -> Self::IntoFuture {
        self.builder.local(self.future).unwrap()
    }
}

impl<Fut> IntoFuture for Builder<Fut, NonLocal>
where
    Fut::Output: Send,
    Fut: Future + Send + 'static,
{
    type Output = Fut::Output;

    type IntoFuture = JoinHandle<Fut::Output>;

    fn into_future(self) -> Self::IntoFuture {
        self.builder.spawn(self.future).unwrap()
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
