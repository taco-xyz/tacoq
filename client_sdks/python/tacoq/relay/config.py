"""Configuration for setting up the relay client.

The user *will* need to configure this manually.
"""

from aiohttp_retry import ExponentialRetry, RetryOptionsBase

from pydantic import BaseModel


class RelayConfig(BaseModel):
    """Configuration for communicating with the relay.

    ### Attributes
    - url: The base URL of the relay (with no paths).
    - retry_options (Optional): The retry options for the publisher's HTTP requests to the relay.
    """

    model_config = {"arbitrary_types_allowed": True}

    url: str
    """ The base URL of the relay (with no paths). """

    ssl_verify: bool = True
    """ If the client should verify the SSL certificate of the request.
    Only disable this when testing in development it is not recommended for production """

    retries: int = 3
    """ The number of retries if failed request """

    timeout: float = 1.0
    """ The timeout between requests on failed request """

    default_retry_options: RetryOptionsBase = ExponentialRetry(
        attempts=3,
        start_timeout=0.2,
        max_timeout=10,
        factor=2.0,
        statuses={500, 502, 503, 504},
    )
    """ The retry options for the publisher's HTTP requests to the relay.
    This can be overriden on a per-request basis.

    Based on [aiohttp_retry](https://github.com/inyutin/aiohttp_retry).
    """
