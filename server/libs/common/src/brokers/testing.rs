use crate::brokers::core::{MockBrokerConsumer, MockBrokerProducer};

pub fn get_mock_broker_producer<T: Send + Sync>() -> MockBrokerProducer<T> {
    MockBrokerProducer::new()
}

pub fn get_mock_broker_consumer<T: Send + Sync>() -> MockBrokerConsumer<T> {
    MockBrokerConsumer::new()
}
