# `stm32f042`

> A basic crate of Rust examples for the STM32F042 (Cortex-M0) microcontroller

# What is this?

STM32F are a very popular (and rather cheap) series of microcontrollers from ST Microelectronics. The [STM32F042](http://www.st.com/content/ccc/resource/technical/document/datasheet/52/ad/d0/80/e6/be/40/ad/DM00105814.pdf/files/DM00105814.pdf/jcr:content/translations/en.DM00105814.pdf) is particularly interesting due to it's availabilty in a hand solderable TSSOP-20 packaging as well as a boatload of features which work without a lot of external parts, such as:
* USB (without external clock!)
* HDMI CEC
* Some 5V tolerant inputs
* Internal ADC
* Capacitive sensor inputs
* CAN
* Built-in RTC

With this crate I hope to offer an easy entry into the world of microcontroller programming with Rust. While there're a lot of very low-level tools available, most of them written by [Jorge Aparicio](http//blog.japaric.io), the current status on the higher levels is _quite_ lacking (to put it in friendly terms).

If you know Arduino, mbed, cmsis, libopencm3 or stm32cube and expect to find something similar in Rust -- the Rust ecosystem for embedded is in a **completely** different ballpark at this stage. While Jorge is actively working on changing that, it's still a long way to go and I do have the feeling that a lot works nicely already but is not really accessible.

Even transfering the really well-done examples by Jorge, e.g. from the [blue-pill board support crate](https://github.com/japaric/blue-pill/) is incredibly painful and requires a medium to full rewrite if you're using anything else but the blue-pill, e.g. the ubiquitious and cheap Nucleo F103RB evaluation board from STM or just use different features.
So I'm trying a different approach here by combining the things that **already do work well** into a very low-level but easy to use crate with a focus on documentation to demonstrate how to easily apply the techniques to a different MCU.

# What is this based on?

As I said I'm trying to re-use all of the useful tools (and ideas) which are available today, most (if not all) of them by Jorge Aparicio. Those are at the moment:

* [xargo](https://github.com/japaric/xargo)
* [svd2rust](https://github.com/japaric/svd2rust)
* [cortex-m](https://github.com/japaric/cortex-m)
* [cortex-m-rt](https://github.com/japaric/cortex-m-rt)

... and a few others required by those mentioned above.

# How to use?

## Prerequisites

* arm-none-eabi gcc toolchain
* rustup (cf. [Rustup homepage](https://www.rustup.rs/))
* xargo: Install via `cargo install xargo`
* rust sources: `rustup component add rust-src`

## How to compile

After you've installed the above prerequisites getting up and running is as simple (as it should be):

`xargo build --release --examples`

This will take a while, especially on first run (also an internet connection needed to download additional crate sources) but will take care of everything and create executables for all rust sources in the **examples** folder.

The compiled binaries (optimized for smallest size) will be available in **target/thumbv6m-none-eabi/release/examples/**

Alternatively you might want to have debugging binaries instead which can be obtained by leaving out the *--release*:

`xargo build --examples`

Those can be picked up from **target/thumbv6m-none-eabi/debug/examples/** instead.

## How to flash

There're two easy ways to flash (and also debug):

### ST-Link

If you have an ST-Link v2 interface, either standalone (original or China mock) or on a STM Nucleo or Discovery board, you can simply hook up your MCU to the ST-Link and use the great [stlink toolset by texane](https://github.com/texane/stlink). To flash the software you can use the `st-flash` executable but you will need to convert the binary by stripping of irrelevant data before, e.g.:

```
# arm-none-eabi-objcopy -O ihex target/thumbv6m-none-eabi/release/examples/blinky blinky.ihex
# st-flash --format ihex write blinky.ihex
```

Debugging is also possible, either using `gdb` or (warning, shameless plug ahead) [`lldb`](https://www.eggers-club.de/blog/2017/07/01/embedded-debugging-with-lldb-sure/).

### Black magic probe

The [Black magic probe](https://1bitsquared.com/collections/frontpage/products/black-magic-probe) is a fantastic piece of hardware made by Piotr Esden-Tempski which is really great for debugging a lot of different MCUs and has a built-in debugging interface (https://github.com/blacksphere/blackmagic) developped as open software, lead by Gareth McMullin. I'd really recommend checking it out.

It is also possible to flash the BMP software onto a cheap ST-Link v2 clone, the ST-Link on a Nucleo or Discovery board or e.g. also the Blue-Pill board, (cf. https://medium.com/@paramaggarwal/converting-an-stm32f103-board-to-a-black-magic-probe-c013cf2cc38c)

The only drawback of using the BMP is that the protocol is incompatible with `lldb` so only `arm-none-eabi-gdb` (yes, that specific one!) can be used for now, e.g.:

```
# arm-none-eabi-gdb target/thumbv6m-none-eabi/release/examples/blinky
...
(gdb) target extended-remote /dev/cu.usbmodemAAAAAAAA
(gdb) monitor swdp_scan
(gdb) attach 1
(gdb) load
(gdb)
```

This will flash the binary into the MCU. You'll have to exchange the device node above by the real one of your device; be sure to use the first one and on a Mac to use the one starting with **/dev/cu** instead of **/dev/tty**.

Note: After the `load` command the MCU will be in a halted state, so you'll either have to use the `continue` command or exit the debugger for the program to run.

# License

Licensed under

- [Creative Commons Attribution 4.0 International Public License](https://creativecommons.org/licenses/by/4.0/)
