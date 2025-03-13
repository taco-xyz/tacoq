import logging
import sys
from typing import Optional, Self


class ColorFormatter(logging.Formatter):
    """Custom color formatter for terminal output"""

    COLORS = {
        "DEBUG": "\033[94m",  # Blue
        "INFO": "\033[92m",  # Green
        "WARNING": "\033[93m",  # Yellow
        "ERROR": "\033[91m",  # Red
        "CRITICAL": "\033[95m",  # Magenta
        "ENDC": "\033[0m",  # Reset
    }

    def format(self: Self, record: logging.LogRecord) -> str:
        if record.levelname in self.COLORS:
            record.levelname = f"{self.COLORS[record.levelname]}{record.levelname}{self.COLORS['ENDC']}"
        return super().format(record)


def setup_logger(name: str = "tacoq", level: Optional[str] = None) -> logging.Logger:
    """Configure and return a logger instance"""
    logger = logging.getLogger(name)

    if not logger.handlers:
        handler = logging.StreamHandler(sys.stderr)
        formatter = ColorFormatter(fmt="%(levelname)s: %(message)s")
        handler.setFormatter(formatter)
        logger.addHandler(handler)

    logger.setLevel(level or logging.INFO)
    return logger


# Create default logger instance
logger = setup_logger()
