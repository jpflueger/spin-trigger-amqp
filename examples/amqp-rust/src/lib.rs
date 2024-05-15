use exports::wasi::messaging::messaging_guest::Guest;
use wasi::messaging::messaging_types::{Error, GuestConfiguration, Message};

spin_messaging_sdk::wit_bindgen::generate!({
    world: "wasi:messaging/messaging",
    path: "../../wit",
    runtime_path: "spin_messaging_sdk::wit_bindgen::rt",
    exports: {
        "wasi:messaging/messaging-guest": MessagingGuest,
    },
    with: {
        "wasi:messaging/messaging-types": spin_messaging_sdk,
    }
});
pub struct MessagingGuest;

impl Guest for MessagingGuest {
    #[doc = r" Returns the list of channels (and extension metadata within guest-configuration) that"]
    #[doc = r" this component should subscribe to and be handled by the subsequent handler within guest-configuration"]
    fn configure() -> Result<GuestConfiguration, Error> {
        Ok(GuestConfiguration {
            channels: Vec::default(),
            extensions: None,
        })
    }

    #[doc = r" Whenever this guest receives a message in one of the subscribed channels, the message is sent to this handler"]
    fn handler(_ms: spin_messaging_sdk::wit_bindgen::rt::vec::Vec<Message>) -> Result<(), Error> {
        println!("Hello, world!");
        Ok(())
    }
}
