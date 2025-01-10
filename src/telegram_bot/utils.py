import logging

import httpx
import httpx as requests

_logger = logging.getLogger(__name__)


def escape_markdown(text: str) -> str:
    reserved_characters = [
        "_",
        "*",
        "[",
        "]",
        "(",
        ")",
        "~",
        "`",
        ">",
        "#",
        "+",
        "-",
        "=",
        "|",
        "{",
        "}",
        ".",
        "!",
    ]
    for reserved in reserved_characters:
        text = text.replace(reserved, rf"\{reserved}")

    return text


class RequestError(Exception):
    pass


def get_json_from_url(url: str, *, headers: dict | None = None) -> dict | None:
    try:
        response = requests.get(url, headers=headers)
        content = response.json()
    except (
        httpx.HTTPError,
        httpx.HTTPStatusError,
    ) as e:
        _logger.exception("failed to communicate with api")
        raise RequestError(e)

    if not response.status_code < 400:
        raise RequestError(f"[{response.status_code}]{response.text}")

    return content
