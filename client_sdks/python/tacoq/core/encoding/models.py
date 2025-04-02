"""Interfaces for byte encoders/decoders."""

from typing import Generic, TypeVar

Data = TypeVar("Data")
""" The type of the data to encode/decode. """


class Encoder(Generic[Data]):
    """An encoder for a specific type of data."""

    def encode(self, data: Data) -> bytes:
        """Encodes an object from the output of the task into bytes."""
        ...


class Decoder(Generic[Data]):
    """A decoder for a specific type of data."""

    def decode(self, data: bytes) -> Data:
        """Decodes an object from bytes into the input of the task."""
        ...


class EncodingError(Exception):
    """Raised when there's an error encoding or decoding a message."""

    pass
