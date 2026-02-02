#![no_std]

#[macro_use]
extern crate alloc;

pub mod config;
pub mod display;
pub mod error;
pub mod graphics;
pub mod network;
pub mod time;
pub mod weather;

// Use https://docs.rs/static_cell/2.1.1/static_cell/macro.make_static.html
// once rust feature(type_alias_impl_trait) is stable
#[macro_export]
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write($val);
        x
    }};
}
