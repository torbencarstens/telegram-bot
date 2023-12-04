import inspect
import os
import socket
import sys
from typing import Dict, Optional

import requests as requests
import urllib3 as urllib3

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


def get_json_from_url(url: str, *, headers: Dict = None) -> Optional[Dict]:
    log = create_logger(inspect.currentframe().f_code.co_name)

    try:
        response = requests.get(url, headers=headers)
        content = response.json()
    except (
        requests.exceptions.ConnectionError,
        socket.gaierror,
        urllib3.exceptions.MaxRetryError,
    ) as e:
        log.exception("failed to communicate with jokes api")
        raise RequestError(e)

    if not response.ok:
        raise RequestError(f"[{response.status_code}]{response.text}")

    return content


def get_env_or_die(env_variable: str, *, exit_code: int = 1) -> str:
    logger = create_logger(inspect.currentframe().f_code.co_name)
    if value := os.getenv(env_variable):
        return value

    logger.critical(f"failed to retrieve token from environment (`{env_variable}`)")
    sys.exit(exit_code)
