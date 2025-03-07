from pydantic import BaseModel
from tacoq.core.infra.broker import BrokerConfig


class WorkerApplicationConfig(BaseModel):
    """Configuration for a worker application. This is passed in on
    initialization of the `WorkerApplication` class.

    ### Attributes:
    - name: The name of the worker. This is used to identify which worker
      executed each task. Make sure to name each worker uniquely!
    - kind: The kind of worker. This dictates which tasks get routed to this
      worker.
    - broker_config: Configuration for the broker. See `BrokerConfig` for more
      details.
    - broker_prefetch_count: The number of tasks to prefetch from the broker.
      This also dictates how many asynchronous tasks can be executed at once.
      This purposefully does not have a default as it is *very* important to set
      it correctly.

    ### Usage
    ```python
    config = WorkerApplicationConfig(
        name="my_worker",
        kind="my_worker",
        broker_config=BrokerConfig(url="amqp://localhost:5672"),
        broker_prefetch_count=10,
        relay_config=RelayConfig(url="http://localhost:8080"),
    )
    ```

    ### Choosing optimal prefetch count
    - If your workload is blocking, the pre-fetch count should be 1 so you can
      guarantee fair dispatch to each worker instance.
    - If your workload is non-blocking, the pre-fetch count should be set to
      the number of concurrent tasks you want to allow at the same time.

    You should also keep in mind that blocking and non-blocking tasks should
    *NOT* be mixed in the same workers as you will end up with tasks blocking
    each other.

    You can read more about prefetch counts in the [RabbitMQ documentation](https://www.rabbitmq.com/docs/confirms#channel-qos-prefetch).
    """

    name: str
    """ The name of the worker. This is used to identify the worker in the relay."""

    kind: str
    """ The kind of worker. This dictates which tasks get routed to this worker."""

    broker_config: BrokerConfig
    """ Configuration for the broker. """

    broker_prefetch_count: int
    """ The number of tasks to prefetch from the broker. This also dictates how 
    many asynchronous tasks can be executed at once. """
