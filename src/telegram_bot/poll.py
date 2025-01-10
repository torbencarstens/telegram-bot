import telegram
from timhatdiehandandermaus_sdk import TimApi

from telegram_bot.config import Config


async def send_movie_poll(*, config: Config, bot: telegram.Bot):
    api = TimApi(api_url=config.api.base_url)
    MAX_POLL_OPTIONS = 10

    question = "Which movie do you want to watch?"
    default_options = ["Mir egal"]

    movie_count = MAX_POLL_OPTIONS - len(default_options)
    movie_options = api.queued_movies(limit=movie_count)
    options = [movie.imdb.title for movie in movie_options]
    options.extend(default_options)

    return await bot.send_poll(
        chat_id=config.telegram.poll_chat_id,
        question=question,
        options=options,
        is_anonymous=False,
        allows_multiple_answers=True,
    )


async def send_participation_poll(*, config: Config, bot: telegram.Bot):
    question = "Ich bin"
    options = ["dabei", "nicht dabei", "vielleicht dabei"]

    return await bot.send_poll(
        chat_id=config.telegram.poll_chat_id,
        question=question,
        options=options,
        is_anonymous=False,
        allows_multiple_answers=False,
    )
