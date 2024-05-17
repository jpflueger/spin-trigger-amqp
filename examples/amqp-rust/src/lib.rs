use exports::wasi::messaging::messaging_guest::Guest;
use wasi::messaging::messaging_types::{Error, GuestConfiguration, Message};

wit_bindgen::generate!({
    world: "wasi:messaging/messaging@0.2.0-draft",
    path: "../../wit",
});
pub struct MessagingComponent;

impl Guest for MessagingComponent {
    fn configure() -> Result<GuestConfiguration, Error> {
        Ok(GuestConfiguration {
            channels: Vec::default(),
            extensions: None,
        })
    }

    fn handler(_ms: Vec<Message>) -> Result<(), Error> {
        println!("Hello, world!");
        Ok(())
    }
}

export!(MessagingComponent);