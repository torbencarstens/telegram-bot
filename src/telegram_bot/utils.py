import inspect
import os
import sys
from typing import Optional

import httpx
import httpx as requests

from logger import create_logger


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


def get_json_from_url(url: str, *, headers: dict | None = None) -> Optional[dict]:
    log = create_logger(inspect.currentframe().f_code.co_name)  # type: ignore

    try:
        response = requests.get(url, headers=headers)
        content = response.json()
    except (
        httpx.HTTPError,
        httpx.HTTPStatusError,
    ) as e:
        log.exception("failed to communicate with api")
        raise RequestError(e)

    if not response.status_code < 400:
        raise RequestError(f"[{response.status_code}]{response.text}")

    return content


def get_env_or_die(env_variable: str, *, exit_code: int = 1) -> str:
    logger = create_logger(inspect.currentframe().f_code.co_name)  # type: ignore
    if value := os.getenv(env_variable):
        return value

    logger.critical(f"failed to retrieve token from environment (`{env_variable}`)")
    sys.exit(exit_code)
