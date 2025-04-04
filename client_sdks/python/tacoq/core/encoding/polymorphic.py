"""Factory functions for creating encoders and decoders based on type."""

from inspect import isclass
from typing import Any, Type, TypeVar, Union, cast, get_origin

from pydantic import BaseModel
from tacoq.core.encoding.json_dict import JsonDictDecoder, JsonDictEncoder
from tacoq.core.encoding.models import Decoder, Encoder, EncodingError
from tacoq.core.encoding.passthrough import PassthroughDecoder, PassthroughEncoder
from tacoq.core.encoding.pydantic import PydanticDecoder, PydanticEncoder
from tacoq.core.encoding.string import StringDecoder, StringEncoder

SUPPORTED_DATA_TYPES = (bytes, str, dict, list, BaseModel)

T = TypeVar("T", bound=Union[bytes, str, dict[str, Any], list[Any], BaseModel])


def create_encoder(data_type: Type[T]) -> Encoder[T]:
    """Creates an appropriate encoder based on the input type.

    ### Arguments:
    - data_type: The type to create an encoder for

    ### Returns:
    An encoder instance appropriate for the given type

    ### Raises:
    - EncodingError: If no suitable encoder is found for the type
    """

    if data_type is bytes:
        return cast(Encoder[T], PassthroughEncoder())
    elif data_type is str:
        return cast(Encoder[T], StringEncoder())
    elif data_type is dict or get_origin(data_type) is dict:
        return cast(Encoder[T], JsonDictEncoder())
    elif isclass(data_type) and issubclass(data_type, BaseModel):
        return cast(Encoder[T], PydanticEncoder())
    else:
        raise EncodingError(
            f"No encoder found for type {data_type}. Available encoder types: {SUPPORTED_DATA_TYPES}. If yours isn't one of these, perhaps you should implement your own encoder?"
        )


def create_decoder(data_type: Type[T]) -> Decoder[T]:
    """Creates an appropriate decoder based on the input type.

    ### Arguments:
    - data_type: The type to create a decoder for

    ### Returns:
    A decoder instance appropriate for the given type

    ### Raises:
    - EncodingError: If no suitable decoder is found for the type
    """

    if data_type is bytes:
        return cast(Decoder[T], PassthroughDecoder())
    elif data_type is str:
        return cast(Decoder[T], StringDecoder())
    elif data_type is dict or get_origin(data_type) is dict:
        return cast(Decoder[T], JsonDictDecoder())
    elif issubclass(data_type, BaseModel):
        return cast(Decoder[T], PydanticDecoder(model=data_type))
    else:
        raise EncodingError(
            f"No decoder found for type {data_type}. Available decoder types: {SUPPORTED_DATA_TYPES}. If yours isn't one of these, perhaps you should implement your own decoder?"
        )
