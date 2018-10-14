stm32f042
=========

_stm32f042_ contains the peripheral access API for the STMicro [stm32f042]
series microcontroller.

The register definitions were created from the collection of CMSIS SVD files at
[cmsis-svd][] with the help of [svd2rust][] to generate the Rust code. 

This crate is now *obsolete*! Use [stm32f0][] instead or have a look at the
[stm32f042-hal][] crate for an implementation using that crate.

[stm32f042]: http://www.st.com/content/st_com/en/products/microcontrollers/stm32-32-bit-arm-cortex-mcus/stm32-mainstream-mcus/stm32f0-series/stm32f0x2/stm32f042f6.html
[cmsis-svd]: https://github.com/posborne/cmsis-svd.git
[svd2rust]: https://github.com/japaric/svd2rust
[stm32f0]: https://crates.io/crates/stm32f0
[stm32f042-hal]: https://crates.io/crates/stm32f042-hal

License
-------

[0-clause BSD license](LICENSE-0BSD.txt).
