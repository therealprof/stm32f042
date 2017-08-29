#![no_std]
#![cfg_attr(feature="rt",feature(global_asm))]
#![cfg_attr(feature="rt",feature(macro_reexport))]
#![cfg_attr(feature="rt",feature(used))]
#![feature(const_fn)]
#![allow(non_camel_case_types)]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate bare_metal;
extern crate vcell;

mod svd;
pub mod peripherals;

pub use svd::*;
pub use cortex_m_rt::*;
