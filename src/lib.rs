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
use std::pin::Pin;
use std::task::{Context, Poll};

use async_std::task;

// /// The `tasky` prelude.
// pub mod prelude {
//     pub use super::IntoFutureExt as _;
// }

/// A handle representing a task.
#[derive(Debug)]
#[pin_project(PinnedDrop)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct JoinHandle<F, Fut>
where
    F: (FnOnce() -> Fut) + Send + 'static,
    Fut: IntoFuture + 'static,
    <Fut as IntoFuture>::Output: Send,
{
    builder: Option<Builder<F, Fut>>,
    #[pin]
    handle: Option<task::JoinHandle<Fut::Output>>,
}

impl<F, Fut> Future for JoinHandle<F, Fut>
where
    F: (FnOnce() -> Fut) + Send + 'static,
    Fut: IntoFuture + 'static,
    <Fut as IntoFuture>::Output: Send,
{
    type Output = <Fut as IntoFuture>::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        if let Some(builder) = this.builder.take() {
            this.handle.replace(
                builder
                    .builder
                    .spawn(task::spawn_local((builder.closure)().into_future()))
                    .unwrap(),
            );
        }
        Pin::new(&mut this.handle.as_pin_mut().unwrap()).poll(cx)
    }
}

/// Cancel a task when dropped.
#[pinned_drop]
impl<F, Fut> PinnedDrop for JoinHandle<F, Fut>
where
    F: (FnOnce() -> Fut) + Send + 'static,
    Fut: IntoFuture + 'static,
    <Fut as IntoFuture>::Output: Send,
{
    fn drop(self: Pin<&mut Self>) {
        let mut this = self.project();
        let handle = this.handle.take().unwrap();
        let _ = handle.cancel();
    }
}

// /// Extend the `Future` trait.
// pub trait IntoFutureExt: IntoFuture + Sized {
//     /// Spawn a task on a thread pool
//     fn spawn<F>(self) -> Builder<F, Self::IntoFuture>
//     where
//         Self: Send,
//     {
//         Builder {
//             kind: PhantomData,
//             closure: self.into_future(),
//             builder: async_std::task::Builder::new(),
//         }
//     }
// }
//
// impl<F> IntoFutureExt for F where F: IntoFuture {}

pub fn spawn<F, Fut>(f: F) -> JoinHandle<F, Fut>
where
    F: (FnOnce() -> Fut) + Send + 'static,
    Fut: IntoFuture + 'static,
    <Fut as IntoFuture>::Output: Send,
{
    let builder = Builder {
        closure: f,
        builder: async_std::task::Builder::new(),
    };

    JoinHandle {
        builder: Some(builder),
        handle: None,
    }
}

/// Sealed trait to determine what type of bulider we got.
mod sealed {
    pub trait Kind {}
}

/// Task builder that configures the settings of a new task.
#[derive(Debug)]
#[must_use = "async builders do nothing unless you call `into_future` or `.await` them"]
pub struct Builder<F, Fut>
where
    F: (FnOnce() -> Fut) + Send + 'static,
    Fut: IntoFuture + 'static,
    <Fut as IntoFuture>::Output: Send,
{
    closure: F,
    builder: async_std::task::Builder,
}

impl<F, Fut> Builder<F, Fut>
where
    F: (FnOnce() -> Fut) + Send + 'static,
    Fut: IntoFuture + 'static,
    <Fut as IntoFuture>::Output: Send,
{
    /// Set the name of the task.
    pub fn name(mut self, name: String) -> Builder<F, Fut> {
        self.builder = self.builder.name(name);
        self
    }
}

impl<F, Fut> IntoFuture for Builder<F, Fut>
where
    F: (FnOnce() -> Fut) + Send + 'static,
    Fut: IntoFuture + 'static,
    <Fut as IntoFuture>::Output: Send,
{
    type Output = Fut::Output;

    type IntoFuture = JoinHandle<F, Fut>;

    fn into_future(self) -> Self::IntoFuture {
        JoinHandle {
            builder: Some(self),
            handle: None,
        }
    }
}

// #[cfg(test)]
// mod test {
//     use super::prelude::*;

//     #[test]
//     fn spawn() {
//         async_std::task::block_on(async {
//             let res = async { "nori is a horse" }.spawn().await;
//             assert_eq!(res, "nori is a horse");
//         })
//     }

//     // #[test]
//     // fn spawn_local() {
//     //     async_std::task::block_on(async {
//     //         let res = async { "nori is a horse" }.spawn_local().await;
//     //         assert_eq!(res, "nori is a horse");
//     //     })
//     // }

//     #[test]
//     fn name() {
//         async_std::task::block_on(async {
//             let res = async { "nori is a horse" }
//                 .spawn()
//                 .name("meow".into())
//                 .await;
//             assert_eq!(res, "nori is a horse");
//         })
//     }
// }
