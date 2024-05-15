use spin_amqp_sdk::{amqp_component, Message};

#[amqp_component]
async fn handler(_messages: Vec<Message>) -> anyhow::Result<()> {
    todo!("")
}