pub use spin_amqp_macro::amqp_component;

#[doc(hidden)]
pub use spin_executor as executor;

#[doc(hidden)]
pub mod wit {
    #![allow(missing_docs)]

    wit_bindgen::generate!({
        world: "spin-amqp-sdk",
        path: "../..",
    });
}

#[doc(hidden)]
pub use wit_bindgen;

#[doc(inline)]
pub use wit::spin::amqp_trigger::spin_amqp_types::{Error, Message, Properties};
