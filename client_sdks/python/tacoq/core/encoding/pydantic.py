"""Encoders/Decoders that translate between Pydantic models and JSON bytes."""

from tacoq.core.encoding.models import Encoder, Decoder, EncodingError
from pydantic import BaseModel
from typing import Type, TypeVar


class PydanticEncoder(Encoder[BaseModel]):
    """Encodes Pydantic models to JSON bytes.

    ### Usage:
    ```python
    from tacoq.core.encoding import PydanticEncoder

    class MyModel(BaseModel):
        name: str
        age: int

    encoder = PydanticEncoder()
    encoded_data = encoder.encode(MyModel(name="John", age=30))
    ```
    """

    def encode(self, data: BaseModel) -> bytes:
        """Default encoder for Pydantic models.

        Serializes a Pydantic model to JSON bytes.

        ### Arguments:
        - data: The Pydantic model to encode

        ### Returns:
        The encoded bytes

        ### Raises:
        - EncodingError: If there's an error encoding the message
        """

        try:
            return data.model_dump_json().encode("utf-8")
        except Exception as e:
            raise EncodingError(f"Error encoding message: {str(e)}")


Model = TypeVar("Model", bound=BaseModel)


class PydanticDecoder(Decoder[Model]):
    """Decodes Pydantic models from JSON bytes based on the input model.

    ### Attributes
    - model: The expected Pydantic model to decode the bytes to.

    ### Usage
    ```python
    from tacoq.core.encoding import PydanticDecoder

    class MyModel(BaseModel):
        name: str
        age: int

    decoder = PydanticDecoder(model=MyModel)
    decoded_data: MyModel = decoder.decode(data)
    ```
    """

    model: Type[Model]

    def __init__(self, model: Type[Model]):
        self.model = model

    def decode(self, data: bytes) -> Model:
        """Default decoder for Pydantic models.

        Deserializes JSON bytes to a Pydantic model.

        ### Arguments:
        - data: The bytes to decode

        ### Returns:
        The decoded Pydantic model

        ### Raises:
        - EncodingError: If there's an error decoding the message
        """

        try:
            return self.model.model_validate_json(data.decode("utf-8"))
        except Exception as e:
            raise EncodingError(f"Error decoding message: {str(e)}")
