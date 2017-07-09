#![feature(asm)]
#![feature(const_fn)]
#![feature(optin_builtin_traits)]
#![no_std]

pub mod stm32f042x;
extern crate vcell;
extern crate cortex_m;
extern crate static_ref;
extern crate volatile_register;
extern crate bare_metal;
