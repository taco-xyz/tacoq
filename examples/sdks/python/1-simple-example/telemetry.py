from tacoq import TracerManager, LoggerManager
from opentelemetry.trace import get_tracer
import logging
import sys

# =============================
# Tracer Managment
# =============================

# You can get the default and current tracer using
# the following methods:
default_tracer = TracerManager.get_default_tracer()
current_tracer = TracerManager.get_tracer()

# You can modify the existing ones or set up an entirely new one
# and inject it as the tracer used by the entire package:
new_tracer = get_tracer(__name__)
TracerManager.set_tracer(new_tracer)


# =============================
# Logger Managment
# =============================

# You can get the default and current logger using
# the following methods:
default_logger = LoggerManager.get_default_logger()
current_logger = LoggerManager.get_logger()

# You can modify the existing ones or set up an entirely new one
# and inject it as the logger used by the entire package.
#
# That being said, it is recommended you instead mofiy the
# default logger as it already contains some TacoQ-specific
# filters and handlers. Read `LoggerManager` for understand
# what they are.

# NOTE - This StreamHandler is included by default, this
# is simply an example!
default_logger.addHandler(logging.StreamHandler(sys.stdout))
