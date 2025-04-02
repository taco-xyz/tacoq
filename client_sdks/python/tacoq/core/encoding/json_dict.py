"""Encoders/Decoders that translate between Python dictionaries and JSON bytes."""

import json
from typing import Dict, Any
from tacoq.core.encoding.models import Encoder, Decoder, EncodingError


class JsonDictEncoder(Encoder[Dict[str, Any]]):
    """Encodes Python dictionaries to JSON bytes.

    ### Usage:
    ```python
    from tacoq.core.encoding import JsonDictEncoder

    encoder = JsonDictEncoder()
    encoded_data = encoder.encode({"name": "John", "age": 30})
    ```
    """

    def encode(self, data: Dict[str, Any]) -> bytes:
        """Encodes a dictionary to JSON bytes.

        ### Arguments:
        - data: The dictionary to encode

        ### Returns:
        The encoded JSON bytes

        ### Raises:
        - EncodingError: If there's an error encoding the message
        """
        try:
            return json.dumps(data).encode("utf-8")
        except Exception as e:
            raise EncodingError(f"Error encoding message: {str(e)}")


class JsonDictDecoder(Decoder[Dict[str, Any]]):
    """Decodes JSON bytes to Python dictionaries.

    ### Usage:
    ```python
    from tacoq.core.encoding import JsonDictDecoder

    decoder = JsonDictDecoder()
    decoded_data = decoder.decode(b'{"name": "John", "age": 30}')
    ```
    """

    def decode(self, data: bytes) -> Dict[str, Any]:
        """Decodes JSON bytes to a dictionary.

        ### Arguments:
        - data: The JSON bytes to decode

        ### Returns:
        The decoded dictionary

        ### Raises:
        - EncodingError: If there's an error decoding the message
        """
        try:
            return json.loads(data.decode("utf-8"))
        except Exception as e:
            raise EncodingError(f"Error decoding message: {str(e)}")
