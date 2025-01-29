from aiohttp_retry import ExponentialRetry, RetryOptionsBase

from pydantic import BaseModel


class ManagerConfig(BaseModel):
    """Configuration for communicating with the manager."""

    model_config = {"arbitrary_types_allowed": True}

    url: str
    """ The base URL of the manager (with no paths). """
    retry_options: RetryOptionsBase = ExponentialRetry(
        attempts=3,
        start_timeout=0.2,
        max_timeout=10,
        factor=2.0,
        statuses={500, 502, 503, 504},
    )
    """ The retry options for the publisher's HTTP requests to the manager.
    This can be overriden on a per-request basis.

    Based on [aiohttp_retry](https://github.com/inyutin/aiohttp_retry).
    """
