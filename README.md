<h1 align="center">parallel-future</h1>
<div align="center">
  <strong>
    structured parallel execution for async Rust
  </strong>
</div>

<br />

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/parallel-future">
    <img src="https://img.shields.io/crates/v/parallel-future.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/parallel-future">
    <img src="https://img.shields.io/crates/d/parallel-future.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/parallel-future">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
</div>

<div align="center">
  <h3>
    <a href="https://docs.rs/parallel-future">
      API Docs
    </a>
    <span> | </span>
    <a href="https://github.com/yoshuawuyts/parallel-future/releases">
      Releases
    </a>
    <span> | </span>
    <a href="https://github.com/yoshuawuyts/parallel-future/blob/master.github/CONTRIBUTING.md">
      Contributing
    </a>
  </h3>
</div>

## Installation
```sh
$ cargo add parallel-future
```

## Examples

```rust
use parallel_future::prelude::*;
use futures_concurrency::prelude::*;

async_std::task::block_on(async {
    let a = async { 1 }.par();        // ← returns `ParallelFuture`
    let b = async { 2 }.par();        // ← returns `ParallelFuture`

    let (a, b) = (a, b).join().await; // ← concurrent `.await`
    assert_eq!(a + b, 3);
})
```

## Safety
This crate uses ``#![deny(unsafe_code)]`` to ensure everything is implemented in
100% Safe Rust.

## Contributing
Want to join us? Check out our ["Contributing" guide][contributing] and take a
look at some of these issues:

- [Issues labeled "good first issue"][good-first-issue]
- [Issues labeled "help wanted"][help-wanted]

[contributing]: https://github.com/yoshuawuyts/parallel-future/blob/master.github/CONTRIBUTING.md
[good-first-issue]: https://github.com/yoshuawuyts/parallel-future/labels/good%20first%20issue
[help-wanted]: https://github.com/yoshuawuyts/parallel-future/labels/help%20wanted

## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br/>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
