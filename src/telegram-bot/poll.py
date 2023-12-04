import os

import telegram
from timhatdiehandandermaus_sdk import TimApi


async def send_movie_poll(*, chat_id: str, bot: telegram.Bot):
    api = TimApi()
    MAX_POLL_OPTIONS = 10

    question = "Which movie do you want to watch?"
    default_options = os.getenv("POLL_DEFAULT_OPTIONS") or "Mir egal"
    default_options = default_options.split(",")

    movie_count = MAX_POLL_OPTIONS - len(default_options)
    movie_options = await api.queued_movies(limit=movie_count)
    options = [movie.imdb.title for movie in movie_options]
    options.extend(default_options)

    return await bot.send_poll(
        chat_id=chat_id,
        question=question,
        options=options,
        is_anonymous=False,
        allows_multiple_answers=True,
    )


async def send_participation_poll(*, chat_id: str, bot: telegram.Bot):
    question = "Ich bin"
    options = ["dabei", "nicht dabei", "vielleicht dabei"]

    return await bot.send_poll(
        chat_id=chat_id,
        question=question,
        options=options,
        is_anonymous=False,
        allows_multiple_answers=False,
    )
