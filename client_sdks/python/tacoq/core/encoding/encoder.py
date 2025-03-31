from typing import Callable, TypeVar, Type
from pydantic import BaseModel

Data = TypeVar("Data")
""" The type of the data to encode/decode. """

EncoderFunction = Callable[[Data], bytes]
""" Encodes an object from the output of the task into bytes. """

DecoderFunction = Callable[[bytes], Data]
""" Decodes an object from bytes into the input of the task. """


def pydantic_encoder(data: BaseModel) -> bytes:
    """Default encoder for Pydantic models.

    Serializes a Pydantic model to JSON bytes.

    ### Arguments:
    - data: The Pydantic model to encode

    ### Returns:
    The encoded bytes
    """
    return data.model_dump_json().encode("utf-8")


def pydantic_decoder(data: bytes, model_class: Type[BaseModel]) -> BaseModel:
    """Default decoder for Pydantic models.

    Deserializes JSON bytes to a Pydantic model.

    ### Arguments:
    - data: The bytes to decode
    - model_class: The Pydantic model class to deserialize into

    ### Returns:
    The decoded Pydantic model
    """
    return model_class.model_validate_json(data.decode("utf-8"))
