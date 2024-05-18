use spin_amqp_sdk::{amqp_component, Message};

#[amqp_component]
async fn handler(messages: Vec<Message>) -> anyhow::Result<()> {
    println!("received {} messages", messages.len());
    for (i, msg) in messages.iter().enumerate() {
        let metadata_str = match &msg.metadata {
            Some(m) => m
                .iter()
                .map(|t| format!("{}={}", t.0, t.1))
                .collect::<Vec<String>>()
                .join(","),
            None => String::new(),
        };
        println!("message {} has length={} and metadata={} ", i+1, msg.data.len(), metadata_str);
    }
    Ok(())
}
