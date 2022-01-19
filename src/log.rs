extern crate web_sys;

#[allow(unused_macros)]
macro_rules! info {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[allow(unused_imports)]
pub(crate) use info;
