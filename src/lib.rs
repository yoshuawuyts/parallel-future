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

mod stream;
mod future;

/// The `tasky` prelude.
pub mod prelude {
    pub use crate::FutureExt as _;
}

pub use future::*;
pub use stream::*;
