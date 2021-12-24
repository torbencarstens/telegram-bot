use std::env;
use std::error::Error;
use std::fmt::{self, Debug};
use std::str::FromStr;

use anyhow::anyhow;
use teloxide::{prelude::*, utils::command::BotCommand};
use teloxide_core::types::PollType;
use tokio;
use tokio_stream::wrappers::UnboundedReceiverStream;
use url::Url;

use timhatdiehandandermaus::api::Api;
use timhatdiehandandermaus::MovieDeleteStatus;

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
    #[command(description = "send poll to choose movie to watch")]
    Poll,
    #[command(description = "noop")]
    Noop,
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
            let url = Url::parse(imdb_url.as_str())
                .map_err(|e| anyhow!("pass a valid URL you idiot: ({:?})", e))?;
            match Command::wade_through("add_movie", api.add_movie(url).await) {
                Ok(value) => value,
                Err(error) => format!("[ add_movie[2]: {:?} ]", error)
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

async fn send_movie_poll(api: Api, bot: &Bot, chat_id: i64) -> anyhow::Result<Message> {
    let question = "Which movie do you want to watch?";

    let mut default_options = env::var("POLL_DEFAULT_OPTIONS")
        .unwrap_or(String::from("Nicht dabei,Mir egal"))
        .split(",")
        .map(|s| s.trim().into())
        .collect::<Vec<String>>();

    let options_count = 10 - default_options.len();
    let mut options = match api.queue().await {
        Ok(value) => value
            .movies
            .into_iter()
            // TODO: throw some kind of error for this
            .filter_map(|item| item.ok())
            .map(|movie| movie.to_string())
            .take(options_count)
            .chain(default_options)
            .collect(),
        Err(err) => {
            bot.send_message(chat_id, format!("Failed to retrieve movies: {:#?}", err));
            vec![]
        }
    };
    if options.len() == 0 {
        let msg = "poll: no movies in queue (or error decoding individual ones)";
        bot.send_message(chat_id, msg);
        Err(anyhow!(msg))?
    }

    Ok(bot
        .send_poll(chat_id, question, options, PollType::Regular)
        .is_anonymous(false)
        .allows_multiple_answers(true)
        .send()
        .await?)
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
        Command::Poll => send_movie_poll(api.clone(), cx.requester.inner(), cx.update.chat_id()).await?,
        Command::Noop => { cx.answer("").await? }
        command_name => {
            cx.answer(format!("{:#?} is not implemented yet.", command_name)).await?
        }
    };

    Ok(())
}

fn parse_command(c: &str) -> anyhow::Result<Command> {
    match BotCommand::parse(c, env::var("BOT_NAME")?) {
        Ok(command) => Ok(command),
        Err(e) => Err(anyhow!("{:?}", e)),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    teloxide::enable_logging!();
    let bot_name = env::var("BOT_NAME")?;
    log::info!("Starting {}...", bot_name);
    let bot = Bot::from_env().auto_send();

    if env::args().any(|arg| arg.to_lowercase() == String::from("poll")) {
        let api = Api::new(env::var("BASE_URL").unwrap_or("http://api".to_string()).parse().expect("BASE_URL is in the wrong format"));
        println!("{:#?}", send_movie_poll(api, bot.inner(), env::var("ADMIN_CHAT").expect("`ADMIN_CHAT` has to be set").parse::<i64>()?).await?);

        return Ok(());
    }

    Dispatcher::new(bot)
        .messages_handler(|rx: DispatcherHandlerRx<AutoSend<Bot>, Message>| {
            UnboundedReceiverStream::new(rx).for_each_concurrent(None, |message| async move {
                let api = Api::new(env::var("BASE_URL").unwrap_or("http://api".to_string()).parse().expect("BASE_URL is in the wrong format"));
                let msg = message.update.clone();
                let command = msg.text().unwrap_or("");
                println!("{:#?}", answer(message, parse_command(command).unwrap_or(Command::Noop), api).await);
            })
        })
        .dispatch()
        .await;

    Ok(())
}
