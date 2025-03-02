from pydantic import BaseModel


class SerializedException(BaseModel):
    """A serialized exception. Transforms the exception into a JSON object that is shared between the manager and the worker.

    ### Example
    ```python
    try:
        raise RuntimeError("test")
    except Exception as e:
        serialized_exception = SerializedException.from_exception(e)
        print(serialized_exception)
    ```

    >>> {"type": "RuntimeError", "message": "test"}
    """

    type: str
    """ The type of the exception. `RuntimeError` evaluates to `"RuntimeError"`."""

    message: str
    """ The message of the exception. """

    @classmethod
    def from_exception(cls, e: Exception):
        return cls(type=e.__class__.__name__, message=str(e))
