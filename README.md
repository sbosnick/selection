# Selection

**Selection provides an embedding of the seL4 microkernal in the Rust ecosystem.**

[![Build Status](https://travis-ci.org/sbosnick/selection.svg?branch=master)](https://travis-ci.org/sbosnick/selection)
---

Selection is an experimental attempt to provide a natural embedding of the building 
of a system based around the seL4 microkernel into the Rust ecosystem. A project
using this embedding will be built using *cargo* and will be able to be bootstrapped
from *U-Boot*.

Selection takes a different approach from that taken by [fel4] in that it attempts
to use *cargo* and Rust idioms, even if this means restricting the available seL4 
options. You should prefer [fel4] until this project matures more.

[fel4]:https://crates.io/crates/cargo-fel4

## Status
Selection is in the early design and implementation stage. Currently the best way
learn more about this project is the [design artifacts][selecton-design]:

 * The requirements: *REQ-*
 * The specifications: *SPC-*
 * The testing descriptions: *TST-*

As the implementation progresses these design artifacts will be supplemented by more
user-oriented documentation.

[selection-design]: https://www.bosnick.ca/selection/

## License

Selection is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE-2.0](LICENSE-APACHE-2.0) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

## Contribution

Please note that this project is released with a [Contributor Code of Conduct][code-of-conduct].
By participating in this project you agree to abide by its terms.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Selection by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[code-of-conduct]: CODE_OF_CONDUCT.md
