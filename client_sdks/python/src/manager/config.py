from dataclasses import dataclass

import aiohttp as aio


@dataclass
class ManagerConfig:
    """Configuration for the manager.

    ### Attributes
    - `url`: The URL of the manager.
    """

    url: str
    timeout: aio.ClientTimeout = aio.ClientTimeout(total=10)
