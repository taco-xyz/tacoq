from logging import Filter, Handler, Logger, LogRecord, getLogger
from threading import Lock
from typing import Optional, Type

from opentelemetry import trace
from opentelemetry.util.types import Attributes
from pydantic import BaseModel, Field
from typing_extensions import Self

## =================================================
## Tracer Processors
## These add OpenTelemetry spans to the event dict
## so we can easily refer to them in logs.
## =================================================


class TraceHandler(Handler):
    """A logging handler that adds OpenTelemetry spans to the event dictionary."""

    def emit(self, record: LogRecord) -> None:
        try:
            StructuredMessage.from_str(record.getMessage()).emit(record.levelno)
        except Exception:
            pass


class TraceFilter(Filter):
    """A logging filter that adds OpenTelemetry spans to the event dictionary."""

    def filter(self, record: LogRecord) -> bool:
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
    """A structured message."""

    message: str
    """ The message to log. """

    attributes: Optional[Attributes] = Field(default_factory=dict)
    """ The attributes to add to the span. """

    def __str__(self) -> str:
        return self.model_dump_json()

    @classmethod
    def from_str(cls, string: str) -> Self:
        """Create a structured message from a string.

        ### Args:
        - string: The string to create the structured message from.

        ### Returns:
        StructuredMessage: The structured message.
        """
        return cls.model_validate_json(string)

    def emit(self, level: int) -> None:
        """Emits itself to the current span."""

        span = trace.get_current_span()
        if not span.is_recording():
            return

        span.add_event(self.message, attributes=self.attributes)


class LoggerManager:
    """Manages the logger for the TacoQ client SDK. Logging for this
    package uses `structlog` for structured logging.

    ### Roll Your Own
    You can inject your own logger using `LoggerManager.set_logger` or use the default one. You can also modify the default one
    using `LoggerManager.get_default_logger`. It is recommended you use structured logging.
    """

    _logger: Optional[Logger] = None
    _logger_lock: Lock = Lock()

    @classmethod
    def get_logger(cls: Type[Self]) -> Logger:
        """Get the logger for the tacoq package. By default

        ### Returns:
        Logger: The logger for the tacoq package. Type is `Any` due to `structlog` returning
        the same and making it impossible to use the correct types when using the default `Logger`.
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

        return logger

    @classmethod
    def set_logger(cls: Type[Self], new_logger: Logger):
        """Manually set the logger for the tacoq package.

        ### Args:
        - new_logger: The logger to use for the tacoq package.
        """
        with cls._logger_lock:
            cls._logger = new_logger
