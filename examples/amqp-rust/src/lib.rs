use spin_amqp_sdk::{amqp_component, Message};

#[amqp_component]
async fn handler(msg: Message) -> anyhow::Result<()> {
    print!("received message of {} bytes on exchange {} with routing key {}", msg.data.len(), msg.exchange, msg.routing_key);
    let metadata_str = match &msg.properties {
        Some(p) => format!("{:#?}", p),
        None => String::new(),
    };
    println!(" and metadata={} ", metadata_str);
    Ok(())
}
