from threading import Lock
from typing import Optional, Type

from opentelemetry import trace
from opentelemetry.trace import Tracer
from typing_extensions import Self

## =================================================
## Tracer Manager
## =================================================


class TracerManager:
    """Manages the tracer for the TacoQ client SDK through a global singleton.

    ### Usage
    You can inject your own tracer using `TracerManager.set_tracer` or use the default one. You can also modify the default one
    using `TracerManager.get_default_tracer`.

    ### Example
    ```python
    from opentelemetry import trace
    from opentelemetry.trace import Tracer

    tracer = Tracer(__name__)
    TracerManager.set_tracer(tracer) # The library now uses your tracer!
    ```
    """

    _tracer: Optional[Tracer] = None
    _tracer_lock: Lock = Lock()

    @classmethod
    def get_tracer(cls: Type[Self]) -> Tracer:
        """Get the tracer for the tacoq package.

        ### Returns:
        Tracer: The tracer for the tacoq package.
        """
        if cls._tracer is None:
            cls.set_tracer(cls.get_default_tracer())
        return cls._tracer  # type: ignore

    @classmethod
    def get_default_tracer(cls: Type[Self]) -> Tracer:
        """Get the default tracer for the tacoq package.

        ### Returns:
        Tracer: The default tracer for the tacoq package.
        """
        return trace.get_tracer("tacoq.python_client")

    @classmethod
    def set_tracer(cls: Type[Self], new_tracer: Tracer):
        """Manually set the tracer for the tacoq package.

        ### Arguments:
        - new_tracer: The tracer to use for the tacoq package.
        """
        with cls._tracer_lock:
            cls._tracer = new_tracer
