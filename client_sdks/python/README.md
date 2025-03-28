# TacoQ Python SDK

## Structure

```bash

# Source Code
src/
├── core/ # Core Components used by the entire application
│   ├── infra/ # Abstractions on top of core infrastructure
│   │   ├── broker/ # RabbitMQ Wrapper
│   │   └── relay/ # Relay REST API Wrapper
│   ├── models/
│   └── telemetry/ # OTEL Singleton Managers
├── publisher/
│    └── ...
└── worker/
     └── ...

# Test Suite
tests/
├── e2e/
│    └── ...
├── unit/
│    └── ... 
└── conftest.py
```