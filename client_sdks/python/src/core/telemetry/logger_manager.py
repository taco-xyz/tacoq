import sys
from logging import Filter, Handler, Logger, LogRecord, getLogger, StreamHandler
from threading import Lock
from typing import Optional, Type
from typing_extensions import Self

from opentelemetry import trace
from opentelemetry.util.types import Attributes
from pydantic import BaseModel, Field

## =================================================
## Tracer Processors
## These add OpenTelemetry spans to the event dict
## so we can easily refer to them in logs.
## =================================================


class TraceHandler(Handler):
    """A logging handler that adds OpenTelemetry spans to the event dictionary."""

    def emit(self: Self, record: LogRecord) -> None:
        try:
            StructuredMessage.from_str(record.getMessage()).emit(record.levelno)
        except Exception:
            pass


class TraceFilter(Filter):
    """A logging filter that adds OpenTelemetry spans to the event dictionary."""

    def filter(self: Self, record: LogRecord) -> bool:
        span = trace.get_current_span()
        ctx = span.get_span_context()
        if span and ctx.is_valid:
            record.trace_id = ctx.trace_id
            record.span_id = ctx.span_id
        return True


## =================================================
## Logger Manager
## =================================================


class StructuredMessage(BaseModel):
    """A structured log that can emit itself to the current span.

    ### Attributes:
    - message: The message to log.
    - attributes: The attributes to add to the log.
    """

    message: str
    """ The message to log. """

    attributes: Optional[Attributes] = Field(default_factory=dict)
    """ The attributes to add to the log. """

    def __str__(self: Self) -> str:
        return self.model_dump_json()

    @classmethod
    def from_str(cls: Type[Self], string: str) -> Self:
        """Create a structured message from a string.

        ### Arguments:
        - string: The string to create the structured message from.

        ### Returns:
        StructuredMessage: The structured message.
        """
        return cls.model_validate_json(string)

    def emit(self: Self, level: int) -> None:
        """Emits itself to the current span."""

        span = trace.get_current_span()
        if not span.is_recording():
            return

        span.add_event(self.message, attributes=self.attributes)


class LoggerManager:
    """Manages the logger for the TacoQ client SDK through a global singleton.

    ### Usage
    You can inject your own logger using `LoggerManager.set_logger` or use the default one. You can also modify the default one
    using `LoggerManager.get_default_logger`. It is recommended you use structured logging.

    ### Example
    Rolling your own logger:
    ```python
    from logging import getLogger

    logger = getLogger(__name__)
    LoggerManager.set_logger(logger) # The library now uses your logger!
    ```

    This approach isn't necessarily recommended for everyone because TacoQ's
    logging system is designed to be attached to the OpenTelemetry spans. You
    can view `get_default_logger` for an example of how to do this, or you can
    instead use the `LoggerManager.get_logger` method to get the current logger
    and modify it:

    ```python
    logger = LoggerManager.get_logger()
    logger.addHandler(MyCustomHandler()) # The library is now using your custom handler on top of its own!
    ```
    """

    _logger: Optional[Logger] = None
    _logger_lock: Lock = Lock()

    @classmethod
    def get_logger(cls: Type[Self]) -> Logger:
        """Get the logger for the tacoq package. By default

        ### Returns:
        Logger: The logger for the tacoq package.
        """
        if cls._logger is None:
            cls.set_logger(cls.get_default_logger())
        return cls._logger  # type: ignore

    @classmethod
    def get_default_logger(cls: Type[Self]) -> Logger:
        """Get the default logger for the tacoq package.

        ### Returns:
        Logger: The default logger for the tacoq package.
        """

        logger = getLogger("tacoq")

        # Add a trace handler to emit spans
        logger.addHandler(TraceHandler())

        # Add a stream handler to log to stdout
        logger.addHandler(StreamHandler(sys.stdout))

        return logger

    @classmethod
    def set_logger(cls: Type[Self], new_logger: Logger):
        """Manually set the logger for the tacoq package.

        ### Arguments:
        - new_logger: The logger to use for the tacoq package.
        """
        with cls._logger_lock:
            cls._logger = new_logger
