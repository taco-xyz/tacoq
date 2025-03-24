"""Base model for all Avro serializable models.

Helps converting between Avro bytes and Pydantic models.
"""

from typing import Any, Type, Self, Callable, TypeVar

from fastavro import parse_schema, schemaless_writer, schemaless_reader  # type: ignore
from fastavro.types import Schema  # type: ignore
from io import BytesIO
from pydantic import BaseModel
import json
import os


class AvroSerializableBaseModel(BaseModel):
    """Base model for all Avro serializable models."""

    @classmethod
    def get_avro_schema(cls) -> Schema:
        """The Avro schema for the model."""

        schema = SCHEMA_CACHE.get(cls)
        if schema is None:
            raise ValueError(f"Avro schema path for {cls.__name__} must be set!")

        return schema

    # Writing

    @property
    def avro_bytes(self: Self) -> bytes:
        """The Avro bytes for the model."""
        buf = BytesIO()
        model_dict = self.model_dump()
        schemaless_writer(buf, self.get_avro_schema(), model_dict)
        return buf.getvalue()

    # Reading

    @classmethod
    def from_avro_bytes(cls: Type[Self], data: bytes) -> Self:
        """Create a model from Avro bytes.

        ### Arguments:
        - data: The Avro bytes to create the task from.

        ### Returns:
        The model created from the Avro bytes.
        """

        buf = BytesIO(data)
        data_dict: dict[str, Any] = schemaless_reader(  # type: ignore
            buf, cls.get_avro_schema(), reader_schema=cls.get_avro_schema()
        )
        return cls(**data_dict)


# Cache of Avro schemas for each model
SCHEMA_CACHE: dict[Type[AvroSerializableBaseModel], Schema] = {}

T = TypeVar("T", bound=AvroSerializableBaseModel)


def avro_schema_path(path: str) -> Callable[[Type[T]], Type[T]]:
    """Decorator that loads and caches the Avro schema for a model.

    ### Arguments:
    - path: Path to the Avro schema file
    """

    def decorator(cls: Type[T]) -> Type[T]:
        # Set schema path relative to current directory
        local_path = os.path.join(os.path.dirname(__file__), path)

        if not os.path.exists(local_path):
            raise ValueError(f"Schema file not found: {local_path}")

        with open(local_path) as f:
            schema_dict = json.load(f)

        SCHEMA_CACHE[cls] = parse_schema(schema_dict)
        return cls

    return decorator
