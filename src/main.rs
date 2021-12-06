use std::env;
use std::error::Error;
use std::fmt::{self, Debug};
use std::str::FromStr;

use anyhow::anyhow;
use teloxide::{prelude::*, utils::command::BotCommand};
use tokio;
use url::Url;

use timhatdiehandandermaus::api::Api;
use timhatdiehandandermaus::{Movie, MovieDeleteStatus};

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
    #[command(description = "add movie to queue (`/add {imdb-link}`)")]
    Add(String),
    #[command(description = "deletes movie from queue (`/delete {title}`)")]
    Delete(String),
    #[command(description = "lists all movies")]
    Queue,
    #[command(description = "rate movie (`/rate {id} {rating}`), rating can be a number between 0 - 10")]
    Rate(CommandTypeMovieRating),
    #[command(description = "remove rating from movie (`/unrate {id}`)")]
    Unrate(String),
    #[command(description = "mark a movie as watched, this deletes the movie from the queue (`/watch {title}`)")]
    Watch(String),
    #[command(description = "get a movie by title (has to be exact) (`/get {title}`)")]
    Get(String),
}

impl Command {
    fn wade_through<T: Debug + fmt::Display>(s: &str, r: anyhow::Result<anyhow::Result<T>>) -> anyhow::Result<String> {
        match r {
            Ok(value) => match value {
                Ok(value) => Ok(format!("{}", value)),
                Err(error) => Err(anyhow!("[ {}[0]: failed decoding body: {:?} ]", s, error))
            }
            Err(error) => Err(anyhow!("[ {}[1]: failed making api request: {:?} ]", s, error))
        }
    }

    pub async fn add_movie(cx: UpdateWithCx<AutoSend<Bot>, Message>, api: Api, imdb_url: String) -> Result<Message, anyhow::Error> {
        let msg = if imdb_url == "" {
            "`/add` must be followed by an imdb URL you idiot".to_string()
        } else {
            let url = Url::parse(imdb_url.as_str());
            if url.is_err() {
                format!("pass a valid URL you idiot: ({:?})", url.err().expect("impossible since `is_err` is checked before unwrapping `err()`"))
            } else {
                let url = url.expect("impossible since `!is_err()` was validated beforehand");

                match Command::wade_through("add_movie", api.add_movie(url).await) {
                    Ok(value) => value,
                    Err(error) => format!("[ add_movie[2]: {:?} ]", error)
                }
            }
        };

        cx
            .answer(msg)
            .await
            .map_err(|e| anyhow!("[ add_movie[3]: couldn't answer: {:?} ]", e))
    }

    pub async fn delete_movie(cx: UpdateWithCx<AutoSend<Bot>, Message>, api: Api, title: String, status: MovieDeleteStatus) -> anyhow::Result<Message> {
        let msg = if title == "" {
            "`/(delete|watch)` must be followed by a movie title you idiot".to_string()
        } else {
            match api.get_movie_by_title(&title).await? {
                // TODO: also search through /movie (needs support from the api)
                None => format!("couldn't find '{}' in queue", title),
                Some(movie) => {
                    match Command::wade_through("delete_movie", api.delete_movie(movie.id, status).await) {
                        Ok(value) => value,
                        Err(error) => format!("[ delete_movie[2]: {:?} ]", error)
                    }
                }
            }
        };

        cx
            .answer(msg)
            .await
            .map_err(|e| anyhow!("[ delete_movie[3]: couldn't answer: {:?} ]", e))
    }

    pub async fn queue(cx: UpdateWithCx<AutoSend<Bot>, Message>, api: Api) -> anyhow::Result<Message> {
        let msg = match Command::wade_through("queue", Ok(api
            .queue()
            .await)) {
            Ok(value) => value,
            Err(error) => format!("[ queue[2]: {:?} ]", error)
        };

        cx
            .answer(msg)
            .await
            .map_err(|e| anyhow!("[ queue[3]: couldn't answer: {:?} ]", e))
    }

    pub async fn get(cx: UpdateWithCx<AutoSend<Bot>, Message>, api: Api, title: String) -> anyhow::Result<Message> {
        let msg = match api.
            get_movie_by_title(&title)
            .await {
            Ok(value) => match value {
                None => format!("{} couldn't be found in queue", title),
                Some(movie) => movie.to_string(),
            },
            Err(error) => format!("[ get[2]: {:?} ]", error)
        };

        cx
            .answer(msg)
            .await
            .map_err(|e| anyhow!("[ get[3]: couldn't answer: {:?} ]", e))
    }
}

async fn answer(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    command: Command,
    api: Api,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    log::info!("{:?}", command);

    // TODO: This should be assembled from members of the group at some point (talk to API for that)
    if cx.update.chat.id != env::var("ADMIN_CHAT")?.parse::<i64>()? {
        cx.answer("You're not allowed to use this bot.").await?;
        log::info!("{} ([u]{:?} | [f]{:?} | [l]{:?} | [t]{:?}) is not allowed to use this bot", cx.update.chat.id, cx.update.chat.username(), cx.update.chat.first_name(), cx.update.chat.last_name(), cx.update.chat.title());

        return Ok(());
    }

    match command {
        Command::Help => cx.answer(Command::descriptions()).await?,
        Command::WerHatDieHandAnDerMaus => {
            cx.answer(format!("Tim")).await?
        }
        Command::Add(imdb_url) => Command::add_movie(cx, api, imdb_url).await?,
        Command::Delete(title) => Command::delete_movie(cx, api, title, MovieDeleteStatus::Deleted).await?,
        Command::Queue => Command::queue(cx, api).await?,
        Command::Watch(title) => Command::delete_movie(cx, api, title, MovieDeleteStatus::Watched).await?,
        Command::Get(title) => Command::get(cx, api, title).await?,
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
