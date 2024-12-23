from broker.config import BrokerConfig
from broker.core import BrokerClient

# TODO: Import only rabbit if tacoq[amqp] is installed
from broker.rabbitmq import RabbitMQBroker


def create_broker_instance(
    config: BrokerConfig, exchange_name: str, worker_id: str
) -> BrokerClient:
    """Create appropriate broker client based on configuration.

    ### Parameters
    - `config`: Configuration for the broker connection
    - `exchange_name`: Name of the exchange to use
    - `worker_id`: Unique identifier for this worker instance

    ### Returns
    - `BrokerClient`: Configured broker client instance

    ### Raises
    - `ValueError`: If broker URL scheme is not supported
    """

    if config.url.startswith("amqp"):
        return RabbitMQBroker(config, exchange_name, worker_id)
    else:
        raise ValueError(f"Unsupported broker URL: {config.url}")
