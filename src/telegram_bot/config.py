from dataclasses import dataclass
from typing import Self

from bs_config import Env


@dataclass(frozen=True, kw_only=True)
class TelegramConfig:
    poll_chat_id: int
    token: str

    @classmethod
    def from_env(cls, env: Env) -> Self:
        return cls(
            poll_chat_id=env.get_int("POLL_CHAT_ID", required=True),
            token=env.get_string("TOKEN", required=True),
        )


@dataclass(frozen=True, kw_only=True)
class Config:
    api_token: str
    app_version: str
    telegram: TelegramConfig
    sentry_dsn: str | None

    @classmethod
    def from_env(cls, env: Env) -> Self:
        return cls(
            api_token=env.get_string("API_TOKEN", required=True),
            app_version=env.get_string("APP_VERSION", default="dev"),
            telegram=TelegramConfig.from_env(env.scoped("TELEGRAM_")),
            sentry_dsn=env.get_string("SENTRY_DSN"),
        )


def load_config() -> Config:
    env = Env.load(include_default_dotenv=True)
    return Config.from_env(env)
