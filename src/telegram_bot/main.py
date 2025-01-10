import asyncio
import logging
import sys

import sentry_sdk
import telegram.ext
from telegram.ext import Application, ApplicationBuilder, filters
from timhatdiehandandermaus_sdk import TimApi

from telegram_bot import bot, poll
from telegram_bot.config import ApiConfig, Config, load_config

_logger = logging.getLogger(__name__)


def main(
    application: Application,
    api_config: ApiConfig,
) -> None:
    # tbot: telegram.Bot = application.bot
    bot.api = TimApi(api_config.token, api_url=api_config.base_url)

    # configure bot
    # asyncio.ensure_future(tbot.set_my_commands(config.COMMANDS))
    # asyncio.ensure_future(tbot.set_my_name(config.NAME))
    # asyncio.ensure_future(tbot.set_my_description(config.DESCRIPTION))
    # asyncio.ensure_future(tbot.set_my_short_description(config.SHORT_DESCRIPTION))

    application.add_handler(
        telegram.ext.CommandHandler(
            "werhatdiehandandermaus", bot.werhatdiehandandermaus
        )
    )
    not_edited_message_filter = ~filters.UpdateType.EDITED_MESSAGE
    application.add_handler(
        telegram.ext.CommandHandler("add", bot.add, filters=not_edited_message_filter)
    )
    application.add_handler(
        telegram.ext.CommandHandler(
            "delete", bot.delete, filters=not_edited_message_filter
        )
    )
    application.add_handler(
        telegram.ext.CommandHandler(
            "watch", bot.watch, filters=not_edited_message_filter
        )
    )
    application.add_handler(
        telegram.ext.CommandHandler(
            "queue", bot.queue, filters=not_edited_message_filter
        )
    )
    application.add_handler(
        telegram.ext.CommandHandler(
            "wostream", bot.wostream, filters=not_edited_message_filter
        )
    )

    # noinspection PyTypeChecker
    application.add_error_handler(bot.error_handler)  # type: ignore

    _logger.info("Starting up")
    application.run_polling()


def _setup_monitoring(config: Config) -> None:
    logging.basicConfig()

    logging.root.level = logging.WARNING
    logging.getLogger(__package__).level = logging.DEBUG

    sentry_dsn = config.sentry_dsn
    if sentry_dsn is not None:
        sentry_sdk.init(
            dsn=sentry_dsn,
            release=config.app_version,
        )
    else:
        _logger.warning("Sentry is disabled")


if __name__ == "__main__":
    config = load_config()
    _setup_monitoring(config)
    _application = ApplicationBuilder().token(config.telegram.token).build()

    args = sys.argv[1:]
    _logger.info("args: %s", args)
    if not args:
        main(_application, config.api)
    else:
        if args[0] == "poll":
            asyncio.run(
                poll.send_movie_poll(
                    config=config,
                    bot=_application.bot,
                )
            )
        elif args[0] == "participation-poll":
            asyncio.run(
                poll.send_participation_poll(
                    config=config,
                    bot=_application.bot,
                )
            )
