"""Generic passthrough decoders that you can use if you need your encoding
logic to be more complex than what can fit within the standard `Encoder`
and `Decoder` interfaces.
"""

from tacoq.core.encoding.models import Encoder, Decoder


class PassthroughEncoder(Encoder[bytes]):
    """Passthrough encoder that returns the input bytes unchanged.

    ### Usage:
    ```python
    from tacoq.core.encoding import PassthroughEncoder

    encoder = PassthroughEncoder()
    encoded_data = encoder.encode(b"Hello, world!")
    # encoded_data will be b"Hello, world!"
    ```
    """

    def encode(self, data: bytes) -> bytes:
        """Passthrough encoder that returns the input bytes unchanged.

        ### Arguments:
        - data: The bytes to encode

        ### Returns:
        The same bytes unchanged
        """
        return data


class PassthroughDecoder(Decoder[bytes]):
    """Passthrough decoder that returns the input bytes unchanged.

    ### Usage:
    ```python
    from tacoq.core.encoding import PassthroughDecoder

    decoder = PassthroughDecoder()
    decoded_data = decoder.decode(b"Hello, world!")
    # decoded_data will be b"Hello, world!"
    ```
    """

    def decode(self, data: bytes) -> bytes:
        """Passthrough decoder that returns the input bytes unchanged.

        ### Arguments:
        - data: The bytes to decode

        ### Returns:
        The same bytes unchanged
        """
        return data
