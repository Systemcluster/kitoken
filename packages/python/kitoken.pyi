from typing import Optional


class Kitoken:
    """
    Kitoken tokenizer.
    A fast and versatile tokenizer for language models.
    """

    def __init__(self, data: bytes) -> None:
        """
        Initializes the tokenizer from a serialized `kitoken` definition.

        :param data: The serialized definition.
        """
        ...

    @staticmethod
    def from_file(path: str) -> None:
        """
        Deserializes the tokenizer definition from a file and initializes the tokenizer.

        :param path: The path to the file.
        """
        ...

    def encode(self, text: str, encode_specials: Optional[bool] = False) -> list[int]:
        """
        Encodes the given text into a sequence of tokens.
        If `encode_specials` is `True`, the text is first split around special tokens which are separately encoded with the special encoder.
        Returns a list of tokens, or an error if no token for a part exists in the encoder and no unknown token id is set in the configuration.

        :param text: The text to encode.
        :param encode_specials: Whether to encode special tokens.
        """
        ...

    def decode(self, data: list[int], decode_specials: Optional[bool] = False) -> bytes:
        """
        Decodes the given sequence of tokens into text.
        Returns a list of bytes, or an error if no byte sequence for a token exists in the decoder and no unknown token is set in the configuration.

        :param data: The sequence of tokens to decode.
        """
    ...

    @staticmethod
    def from_sentencepiece(data: bytes) -> Kitoken:
        """
        Initializes the tokenizer from a serialized `sentencepiece` model.

        :param data: The serialized model.
        """
        ...

    @staticmethod
    def from_sentencepiece_file(path: str) -> Kitoken:
        """
        Initializes the tokenizer from a `sentencepiece` model file.

        :param path: The path to the file.
        """
        ...

    @staticmethod
    def from_tiktoken(data: bytes) -> Kitoken:
        """
        Initializes the tokenizer from a serialized `tiktoken` model.

        :param data: The serialized model.
        """
        ...

    @staticmethod
    def from_tiktoken_file(path: str) -> Kitoken:
        """
        Initializes the tokenizer from a `tiktoken` model file.

        :param path: The path to the file.
        """
        ...

    @staticmethod
    def from_tokenizers(data: bytes) -> Kitoken:
        """
        Initializes the tokenizer from a serialized `tokenizers` model.

        :param data: The serialized model.
        """
        ...

    @staticmethod
    def from_tokenizers_file(path: str) -> Kitoken:
        """
        Initializes the tokenizer from a `tokenizers` model file.

        :param path: The path to the file.
        """
        ...

    @staticmethod
    def from_tekken(data: bytes) -> Kitoken:
        """
        Initializes the tokenizer from a serialized `tekken` model.

        :param data: The serialized model.
        """
        ...

    @staticmethod
    def from_tekken_file(path: str) -> Kitoken:
        """
        Initializes the tokenizer from a `tekken` model file.

        :param path: The path to the file.
        """
        ...
