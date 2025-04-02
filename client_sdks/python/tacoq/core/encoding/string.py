"""Encoders/Decoders that translate between strings and bytes."""

from tacoq.core.encoding.models import Encoder, Decoder, EncodingError


class StringEncoder(Encoder[str]):
    """Encodes strings to bytes.

    ### Usage:
    ```python
    from tacoq.core.encoding import StringEncoder

    encoder = StringEncoder()
    encoded_data = encoder.encode("Hello, world!")
    ```
    """

    def encode(self, data: str) -> bytes:
        """Encodes a string to bytes.

        ### Arguments:
        - data: The string to encode

        ### Returns:
        The encoded bytes

        ### Raises:
        - EncodingError: If there's an error encoding the message
        """
        try:
            return data.encode("utf-8")
        except Exception as e:
            raise EncodingError(f"Error encoding message: {str(e)}")


class StringDecoder(Decoder[str]):
    """Decodes bytes to strings.

    ### Usage:
    ```python
    from tacoq.core.encoding import StringDecoder

    decoder = StringDecoder()
    decoded_data = decoder.decode(b"Hello, world!")
    ```
    """

    def decode(self, data: bytes) -> str:
        """Decodes bytes to a string.

        ### Arguments:
        - data: The bytes to decode

        ### Returns:
        The decoded string

        ### Raises:
        - EncodingError: If there's an error decoding the message
        """
        try:
            return data.decode("utf-8")
        except Exception as e:
            raise EncodingError(f"Error decoding message: {str(e)}")
