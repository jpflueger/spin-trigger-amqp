use anyhow::anyhow;
use clap::Args;
use futures::StreamExt;
use lapin::options::{BasicAckOptions, BasicConsumeOptions, BasicNackOptions};
use lapin::types::FieldTable;
use lapin::{Connection, ConnectionProperties};
use serde::{Deserialize, Serialize};
use spin_app::MetadataKey;
use spin_core::{async_trait, InstancePre};
use spin_trigger::{TriggerAppEngine, TriggerExecutor};
use std::sync::Arc;

// https://docs.rs/wasmtime/latest/wasmtime/component/macro.bindgen.html
wasmtime::component::bindgen!({
    path: ".",
    world: "spin-amqp",
    async: true,
});

use spin::amqp_trigger::spin_amqp_types as amqp_types;

pub(crate) type RuntimeData = ();
pub(crate) type _Store = spin_core::Store<RuntimeData>;

#[derive(Args)]
pub struct CliArgs {
    /// If true, run each component once and exit
    #[clap(long)]
    pub test: bool,
}

// The trigger structure with all values processed and ready
#[derive(Clone)]
pub struct AmqpTrigger {
    engine: Arc<TriggerAppEngine<Self>>,
    address: String,
    _username: String,
    _password: String,
    component_configs: Vec<(String, String)>,
}

// Application settings (raw serialization format)
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TriggerMetadata {
    r#type: String,
    address: String,
    username: String,
    password: String,
    keep_alive_interval: String,
}

// Per-component settings (raw serialization format)
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AmqpTriggerConfig {
    component: String,
    topic: String,
}

const TRIGGER_METADATA_KEY: MetadataKey<TriggerMetadata> = MetadataKey::new("trigger");

#[async_trait]
impl TriggerExecutor for AmqpTrigger {
    const TRIGGER_TYPE: &'static str = "amqp";
    type RuntimeData = RuntimeData;
    type TriggerConfig = AmqpTriggerConfig;
    type RunConfig = CliArgs;
    type InstancePre = InstancePre<RuntimeData>;

    async fn new(engine: spin_trigger::TriggerAppEngine<Self>) -> anyhow::Result<Self> {
        let address = resolve_template_variable(
            &engine,
            engine.app().require_metadata(TRIGGER_METADATA_KEY)?.address,
        )?;
        let username = resolve_template_variable(
            &engine,
            engine
                .app()
                .require_metadata(TRIGGER_METADATA_KEY)?
                .username,
        )?;
        let password = resolve_template_variable(
            &engine,
            engine
                .app()
                .require_metadata(TRIGGER_METADATA_KEY)?
                .password,
        )?;

        let component_configs =
            engine
                .trigger_configs()
                .try_fold(vec![], |mut acc, (_, config)| {
                    let component = config.component.clone();
                    let topic = resolve_template_variable(&engine, config.topic.clone())?;
                    acc.push((component, topic));
                    anyhow::Ok(acc)
                })?;

        Ok(Self {
            engine: Arc::new(engine),
            address,
            _username: username,
            _password: password,
            component_configs,
        })
    }

    async fn run(self, config: Self::RunConfig) -> anyhow::Result<()> {
        if config.test {
            for component in &self.component_configs {
                let message = Message{
                   data: b"test message".to_vec(),
                   format: amqp_types::FormatSpec::Amqp,
                   metadata: None,
                };
                self.handle_message(&component.0, &[message])
                    .await?;
            }

            Ok(())
        } else {
            tokio::spawn(async move {
                // This trigger spawns threads, which Ctrl+C does not kill. So
                // for this case we need to detect Ctrl+C and shut those threads
                // down. For simplicity, we do this by terminating the process.
                tokio::signal::ctrl_c()
                    .await
                    .expect("failed to listen for Ctrl+C");
                std::process::exit(0);
            });

            let tasks: Vec<_> = self
                .component_configs
                .clone()
                .into_iter()
                .map(|(component_id, topic)| {
                    let trigger = self.clone();
                    tokio::spawn(async move {
                        trigger
                            .run_listener(component_id.as_str(), topic.as_str())
                            .await
                    })
                })
                .collect();

            // wait for the first handle to be returned and drop the rest
            let (result, _, rest) = futures::future::select_all(tasks).await;

            drop(rest);
            result?
        }
    }
}

impl AmqpTrigger {
    async fn handle_message(
        &self,
        component_id: &str,
        message: &[amqp_types::Message],
    ) -> anyhow::Result<()> {
        // Load the guest wasm component
        let (instance, mut store) = self.engine.prepare_instance(component_id).await?;

        // SpinAmqp is auto generated by bindgen as per WIT files referenced above.
        let instance = SpinAmqp::new(&mut store, &instance)?;

        instance
            .call_handler(store, message)
            .await?
            .map_err(|err| anyhow!("failed to execute guest: {err}"))
    }

    async fn run_listener(
        &self,
        component_id: &str,
        topic: &str,
    ) -> anyhow::Result<()> {
        let uri = self.address.clone();
        let conn_opts = ConnectionProperties::default()
            .with_executor(tokio_executor_trait::Tokio::current())
            .with_reactor(tokio_reactor_trait::Tokio)
            .with_connection_name("spin-trigger".into());
        
        //TODO: do we re-use the same channel or create a new channel per requested topic?
        let connection = Connection::connect(&uri, conn_opts)
            .await?;
        let channel = connection.create_channel()
            .await?;

        //TODO: do we need non-defaults for any of these? how do we specify those?
        let consumer_tag = "spin_trigger_0";
        let consume_options = BasicConsumeOptions::default();
        let consume_args = FieldTable::default();

        let mut consumer = channel
            .basic_consume(topic, consumer_tag, consume_options, consume_args)
            .await?;

        //TODO: I don't think I'm using tokio properly...or at all?
        while let Some(delivery) = consumer.next().await {
            // received message
            if let Ok(delivery) = delivery {
                // copy metadata from delivery
                //TODO: are the delivery properties the only thing the consumer needs?
                let mut metadata: Vec<(String, String)> = Default::default();
                if let Some(content_type) = delivery.properties.content_type() {
                    metadata.push(("content_type".to_string(), content_type.to_string()))
                }
                if let Some(content_encoding) = delivery.properties.content_encoding() {
                    metadata.push(("content_encoding".to_string(), content_encoding.to_string()))
                }
                //TODO: how do we handle headers with one metadata array? key prefix or something hacky?
                // if let Some(headers) = delivery.properties.headers() {
                //     metadata.push(("headers".to_string(), headers.to_string()))
                // }
                if let Some(delivery_mode) = delivery.properties.delivery_mode() {
                    metadata.push(("delivery_mode".to_string(), delivery_mode.to_string()))
                }
                if let Some(priority) = delivery.properties.priority() {
                    metadata.push(("priority".to_string(), priority.to_string()))
                }
                if let Some(correlation_id) = delivery.properties.correlation_id() {
                    metadata.push(("correlation_id".to_string(), correlation_id.to_string()))
                }
                if let Some(reply_to) = delivery.properties.reply_to() {
                    metadata.push(("reply_to".to_string(), reply_to.to_string()))
                }
                if let Some(expiration) = delivery.properties.expiration() {
                    metadata.push(("expiration".to_string(), expiration.to_string()))
                }
                if let Some(message_id) = delivery.properties.message_id() {
                    metadata.push(("message_id".to_string(), message_id.to_string()))
                }
                if let Some(timestamp) = delivery.properties.timestamp() {
                    metadata.push(("timestamp".to_string(), timestamp.to_string()))
                }
                if let Some(kind) = delivery.properties.kind() {
                    metadata.push(("kind".to_string(), kind.to_string()))
                }
                if let Some(user_id) = delivery.properties.user_id() {
                    metadata.push(("user_id".to_string(), user_id.to_string()))
                }
                if let Some(app_id) = delivery.properties.app_id() {
                    metadata.push(("app_id".to_string(), app_id.to_string()))
                }
                if let Some(cluster_id) = delivery.properties.cluster_id() {
                    metadata.push(("cluster_id".to_string(), cluster_id.to_string()))
                }

                // create the message
                let message = amqp_types::Message{
                    data: delivery.data.to_owned(),
                    format: amqp_types::FormatSpec::Amqp,
                    metadata: if metadata.is_empty() {None} else {Some(metadata)},
                };

                // send to guest for processing
                let handle_result = self.handle_message(component_id, &[message])
                    .await;

                // ack or nack depending on guest processing
                match handle_result {
                    Ok(()) => delivery
                        .ack(BasicAckOptions::default())
                        .await?,
                    Err(error) => {
                        //TODO: allow the guest to set retries or dead letter queue?
                        dbg!("guest failed to process message: {}", error);
                        delivery
                            .nack(BasicNackOptions{
                                requeue: !delivery.redelivered,
                                ..Default::default()
                            })
                            .await?
                    },
                }
            }
        }

        Ok(())
    }
}

fn resolve_template_variable(
    engine: &TriggerAppEngine<AmqpTrigger>,
    template_string: String,
) -> anyhow::Result<String> {
    let template_expr = spin_expressions::Template::new(template_string)?;
    anyhow::Ok(engine.resolve_template(&template_expr)?)
}
