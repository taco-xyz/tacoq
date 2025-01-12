# Benchmark Suite

To benchmark the performance of the SDK, we use the `pytest-benchmark` library.

To run the benchmarks, use the following command:

```bash
uv run pytest -m "bench"
```

This requires running Postgres, RabbitMQ, and the manager service.
