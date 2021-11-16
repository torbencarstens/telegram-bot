use std::env;
use std::error::Error;
use std::fmt::Debug;
use std::str::FromStr;

use anyhow::anyhow;
use serde::Serialize;
use teloxide::{prelude::*, utils::command::BotCommand};
use tokio;
use url::Url;

use timhatdiehandandermaus::api::Api;

#[derive(Debug)]
struct CommandTypeMovieRating {
    movie_id: String,
    rating: u8,
}

impl FromStr for CommandTypeMovieRating {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(CommandTypeMovieRating::try_from(s.splitn(2, " ").collect::<Vec<&str>>())?)
    }
}

impl TryFrom<Vec<&str>> for CommandTypeMovieRating {
    type Error = anyhow::Error;

    fn try_from(value: Vec<&str>) -> Result<Self, Self::Error> {
        return Ok(CommandTypeMovieRating {
            // TODO: use ok_or
            movie_id: value.first().unwrap().to_string(),
            rating: value.last().unwrap().parse::<u8>()?,
        });
    }
}

#[derive(BotCommand, Debug)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "help")]
    Help,
    #[command(description = "inform the user who has the hand on the mouse")]
    WerHatDieHandAnDerMaus,
    #[command(description = "add movie to queue (`/addmovie {imdb-link}`)")]
    AddMovie(String),
    #[command(description = "deletes movie from queue (`/deletemovie {id}`)")]
    DeleteMovie(String),
    #[command(description = "mark movie as watched from queue (`/deletemovie {id}`)")]
    WatchMovie(String),
    #[command(description = "lists all movies")]
    ListMovies,
    #[command(description = "rate movie (`/ratemovie {id} {rating}`), rating can be a number between 0 - 10")]
    RateMovie(CommandTypeMovieRating),
    #[command(description = "remove rating from movie (`/unratemovie {id}`)")]
    UnrateMovie(String),
}

impl Command {
    fn wade_through<T: Serialize + Debug>(s: &str, r: anyhow::Result<anyhow::Result<anyhow::Result<T>>>) -> anyhow::Result<String> {
        match r {
            Ok(value) => {
                match value {
                    Ok(value) => {
                        match value {
                            Ok(result) => serde_json::to_string(&result)
                                .map_err(|error| anyhow!("[ {}[2]: failed to deserialize movie: {:?}]", error, s)),
                            Err(error) => Err(anyhow!("[ {}[3]: {:?} ]", s, error))
                        }
                    }
                    Err(error) => Err(anyhow!("[ {}[4]: {:?} ]", s, error))
                }
            }
            Err(error) => Err(anyhow!("[ {}[5]: {:?} ]", s, error))
        }
    }

    pub async fn add_movie(cx: UpdateWithCx<AutoSend<Bot>, Message>, api: Api, imdb_url: String) -> Result<Message, anyhow::Error> {
        let msg = if imdb_url == "" {
            "`/addmovie` must be followed by an imdb URL you idiot".to_string()
        } else {
            let url = Url::parse(imdb_url.as_str());
            if url.is_err() {
                format!("pass a valid URL you idiot: ({:?})", url.err().expect("impossible since `is_err` is checked before unwrapping `err()`"))
            } else {
                let url = url.expect("impossible since `!is_err()` was validated beforehand");

                match Command::wade_through("add_movie", api.add_movie(url).await) {
                    Ok(value) => value,
                    Err(error) => format!("[ add_movie[6]: {:?} ]", error)
                }
            }
        };

        cx
            .answer(msg)
            .await
            .map_err(|e| anyhow!("[ add_movie[7]: couldn't answer: {} ]", e))
    }

    pub async fn delete_movie(cx: UpdateWithCx<AutoSend<Bot>, Message>, api: Api, movie_id: String) -> anyhow::Result<Message> {
        let msg = if movie_id == "" {
            "`/deletemovie` must be followed by a movie ID you idiot".to_string()
        } else {
            match Command::wade_through("delete_movie", api.delete_movie(movie_id).await) {
                Ok(value) => value,
                Err(error) => format!("[ delete_movie[6]: {:?} ]", error)
            }
        };

        cx
            .answer(msg)
            .await
            .map_err(|e| anyhow!("[ delete_movie[7]: couldn't answer: {} ]", e))
    }
}

async fn answer(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    command: Command,
    api: Api,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // TODO: This should be assembled from members of the group at some point (talk to API for that)
    if cx.update.chat.id != env::var("ADMIN_CHAT")?.parse::<i64>()? {
        return Ok(cx.answer("You're not allowed to use this bot.").await?);
    }

    match command {
        Command::Help => cx.answer(Command::descriptions()).await?,
        Command::WerHatDieHandAnDerMaus => {
            cx.answer(format!("Tim")).await?
        }
        Command::AddMovie(imdb_url) => Command::add_movie(cx, api, imdb_url).await?,
        Command::DeleteMovie(movie_id) => Command::delete_movie(cx, api, movie_id).await?,
        command_name => {
            cx.answer(format!("{:#?} is not implemented yet.", command_name)).await?
        }
    };

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    teloxide::enable_logging!();
    let bot_name = env::var("BOT_NAME")?;
    log::info!("Starting {}...", bot_name);
    let bot = Bot::from_env().auto_send();

    let api = Api::new(env::var("BASE_URL").unwrap_or("http://api".to_string()).parse()?);

    teloxide::commands_repl(bot, bot_name, move |r, x| answer(r, x, api.clone())).await;

    Ok(())
}
