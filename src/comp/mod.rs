#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - control and status register"]
    pub csr: CSR,
}
#[doc = "control and status register"]
pub struct CSR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "control and status register"]
pub mod csr;
