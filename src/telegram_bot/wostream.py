import os
from collections import defaultdict

import httpx
from pydantic import BaseModel

from timhatdiehandandermaus_sdk.models import MovieResponse
from telegram_bot.utils import escape_markdown


class WostreamSearchMovieResponse(BaseModel):
    title: str
    year: int
    id: int
    type: str

    def telegram_markdown_v2(self) -> str:
        title = escape_markdown(self.title)
        return rf"{title} \({self.year}\)"

    def __hash__(self):
        return self.id


class WostreamSearchResponseProvider(BaseModel):
    name: str
    movies: set[WostreamSearchMovieResponse]

    def telegram_markdown_v2(self) -> str:
        header = f"__{escape_markdown(self.name)}__"
        body = "\n".join([movie.telegram_markdown_v2() for movie in self.movies])

        return "\n".join([header, body])


def _raw(movie: MovieResponse) -> dict | None:
    base_url = os.getenv("WOSTREAM_BASE_URL") or "http://streamingprovider-resolver/search"
    url = "/".join([base_url.rstrip("/"), "search"])

    response = httpx.post(
        url,
        json={"title": movie.imdb.title, "year": movie.imdb.year},
        headers={"Content-Type": "application/json"},
        timeout=60,
    )
    if response.status_code == 404:
        return None

    response.raise_for_status()

    return response.json()


async def search(movie: MovieResponse) -> list[WostreamSearchResponseProvider] | None:
    data = _raw(movie)
    if not data:
        return None

    results = data["results"]
    return [WostreamSearchResponseProvider.model_validate(provider) for provider in results]


async def search_multiple(
    movies: list[MovieResponse],
) -> list[WostreamSearchResponseProvider] | None:
    raw_responses = [_raw(movie) for movie in movies]
    responses = [response for response in raw_responses if response]
    if not responses:
        return None

    providers = defaultdict(list)
    for response in responses:
        results = response["results"]
        for provider in results:
            providers[provider["name"]].extend(provider["movies"])

    responseProviders = [
        {"name": provider_name, "movies": movies} for provider_name, movies in providers.items()
    ]
    return [
        WostreamSearchResponseProvider.model_validate(provider) for provider in responseProviders
    ]
