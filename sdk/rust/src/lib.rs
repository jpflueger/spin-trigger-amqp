pub use spin_messaging_macro::messaging_component;

#[doc(hidden)]
pub use spin_executor as executor;

#[doc(hidden)]
pub mod wit {
    #![allow(missing_docs)]

    wit_bindgen::generate!({
        world: "imports",
        path: "../../wit",
    });
}

#[doc(hidden)]
pub use wit_bindgen;

#[doc(inline)]
pub use wit::wasi::messaging::messaging_types::*;