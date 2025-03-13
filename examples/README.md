
# TacoQ Examples
In this folder, you can read up on examples of using TacoQ for different use cases and languages. Examples are detailed but assume you are already familiar with TacoQ. For an introduction, head over to the [official documentation](github.com) to get started.

## Infrastructure

There are two ways to set up TacoQ's infrastructure:
- **Full Setup**: Involves launching Postgres, RabbitMQ, and the TacoQ relay all in separate containers. Highly recommended for production applications.  Example in `infrastructure/full`.
- **All-in-one**: Great for development environments and for getting started quickly. Example in `infrastructure/all-in-one`.

## SDKs

Currently, TacoQ only supports Python. In the near future, more languages will be supported. TacoQ requires infrastructure to be running so that tasks can get routed through the broker into the workers and stored in the database.

**Examples can be launched with either infrastructure setup unless explicitely specified otherwise.**

### Python SDK

Python's SDK is Pythonic:
- Tasks can be registered via decorators.
- All tasks are represented by async functions.
- Thrown exceptions are automatically serialized into JSON objects.

You can view Python examples in `sdks/python`. Head over to that section's `README.md` to learn what each example contains.

## Observability

Setting up the observability stack for TacoQ is optional, but highly recommended for distributed production applications.

You can view ready-to-be-used examples of Tempo and Grafana in `infrastructure`. The Grafana example also includes some dashboards that you can copy over to your own project.