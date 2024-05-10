# Spin Trigger for wasi-messaging proposal

This trigger directly uses the [wasi-messaging](https://github.com/WebAssembly/wasi-messaging) proposal as a trigger definition with a goal of incrementally implementing different formats.

## MVP

To get started we'll target supporting AMQP 0.9.1 using the lapin library along with basic sdk functionality.

- [ ] Add host implementation for AMQP 0.9.1 using lapin
- [ ] Make usable through an sdk crate
- [ ] Add an example that uses local RabbitMQ

## Next up

- [ ] Implement MQTT, adopting the spinkube trigger
- [ ] Implement Kafka

## Questions
- [ ] Implement raw (is this just TCP frames or something?)
- [ ] How do we support both AMQP 0.9.1 and AMQP 1.0? 
- [ ] Implement cloudevents (is this just an http listener?)
- [ ] Implement http (should we implement this or just use the http trigger?)
