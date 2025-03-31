import pytest
from tacoq.core.encoding.passthrough import PassthroughDecoder, PassthroughEncoder
from tacoq.core.encoding.pydantic import PydanticDecoder, PydanticEncoder
from tests.conftest import TestInputPydanticModel


@pytest.mark.unit
def test_pydantic_encoder_decoder_roundtrip():
    # Create a test model instance
    original_model = TestInputPydanticModel(value=5)

    # Encode and then decode
    encoder = PydanticEncoder()
    decoder = PydanticDecoder(TestInputPydanticModel)

    encoded_data = encoder.encode(original_model)
    decoded_model = decoder.decode(encoded_data)

    # Verify the roundtrip preserves all data
    assert decoded_model == original_model


@pytest.mark.unit
def test_passthrough_encoder_decoder_roundtrip():
    # Create test bytes
    original_data = b"Hello, world!"

    # Encode and then decode
    encoder = PassthroughEncoder()
    decoder = PassthroughDecoder()

    encoded_data = encoder.encode(original_data)
    assert original_data == encoded_data

    decoded_data = decoder.decode(encoded_data)
    assert decoded_data == original_data
