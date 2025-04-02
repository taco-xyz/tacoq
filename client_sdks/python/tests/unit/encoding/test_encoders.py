import pytest
from typing import Any
from tacoq.core.encoding.passthrough import PassthroughDecoder, PassthroughEncoder
from tacoq.core.encoding.pydantic import PydanticDecoder, PydanticEncoder
from tacoq.core.encoding.json_dict import JsonDictDecoder, JsonDictEncoder
from tacoq.core.encoding.string import StringDecoder, StringEncoder
from tacoq.core.encoding.polymorphic import create_encoder, create_decoder
from tacoq.core.encoding.models import EncodingError
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


@pytest.mark.unit
def test_json_dict_encoder_decoder_roundtrip():
    # Create test dictionary
    original_data = {"key": "value", "number": 42, "nested": {"foo": "bar"}}

    # Encode and then decode
    encoder = JsonDictEncoder()
    decoder = JsonDictDecoder()

    encoded_data = encoder.encode(original_data)
    decoded_data = decoder.decode(encoded_data)

    # Verify the roundtrip preserves all data
    assert decoded_data == original_data


@pytest.mark.unit
def test_string_encoder_decoder_roundtrip():
    # Create test string
    original_data = "Hello, world! 🚀"

    # Encode and then decode
    encoder = StringEncoder()
    decoder = StringDecoder()

    encoded_data = encoder.encode(original_data)
    decoded_data = decoder.decode(encoded_data)

    # Verify the roundtrip preserves all data
    assert decoded_data == original_data


@pytest.mark.unit
def test_polymorphic_encoder_decoder_roundtrip():
    # Test string encoding/decoding
    string_encoder = create_encoder(str)
    string_decoder = create_decoder(str)
    string_data = "Hello"
    assert string_decoder.decode(string_encoder.encode(string_data)) == string_data

    # Test dict encoding/decoding
    dict_encoder = create_encoder(dict[str, Any])
    dict_decoder = create_decoder(dict[str, Any])
    dict_data = {"key": "value"}
    assert dict_decoder.decode(dict_encoder.encode(dict_data)) == dict_data

    # Test pydantic encoding/decoding
    pydantic_encoder = create_encoder(TestInputPydanticModel)
    pydantic_decoder = create_decoder(TestInputPydanticModel)
    pydantic_data = TestInputPydanticModel(value=42)
    assert (
        pydantic_decoder.decode(pydantic_encoder.encode(pydantic_data)) == pydantic_data
    )

    # Test bytes encoding/decoding
    bytes_encoder = create_encoder(bytes)
    bytes_decoder = create_decoder(bytes)
    bytes_data = b"Hello, world!"
    assert bytes_decoder.decode(bytes_encoder.encode(bytes_data)) == bytes_data

    # Test unsupported data type
    with pytest.raises(EncodingError) as exc_info:
        create_encoder(int)  # type: ignore
    assert "No encoder found for type" in str(exc_info.value)

    with pytest.raises(EncodingError) as exc_info:
        create_decoder(int)  # type: ignore
    assert "No decoder found for type" in str(exc_info.value)
