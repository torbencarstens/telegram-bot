use std::env;
use std::fmt::{self, Debug};
use std::str::FromStr;

use anyhow::anyhow;
use teloxide::{prelude2::*, utils::command::BotCommand};
use teloxide_core::types::{InlineQuery, InlineQueryResultArticle, InputMessageContent, InputMessageContentText};
use tokio;
use url::Url;

use timhatdiehandandermaus::api::Api;
use timhatdiehandandermaus::MovieDeleteStatus;

static POLL_MAX_OPTIONS_COUNT: usize = 10;

#[derive(Clone, Debug)]
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

#[derive(BotCommand, Clone, Debug)]
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

    pub async fn add_movie(bot: AutoSend<Bot>, message: Message, api: Api, imdb_url: String) -> Result<Message, anyhow::Error> {
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

        bot
            .send_message(message.chat.id, msg)
            .await
            .map_err(|e| anyhow!("[ add_movie[3]: couldn't answer: {:?} ]", e))
    }

    pub async fn delete_movie(bot: AutoSend<Bot>, message: Message, api: Api, title: String, status: MovieDeleteStatus) -> anyhow::Result<Message> {
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

        bot
            .send_message(message.chat.id, msg)
            .await
            .map_err(|e| anyhow!("[ delete_movie[3]: couldn't answer: {:?} ]", e))
    }

    pub async fn queue(bot: AutoSend<Bot>, message: Message, api: Api) -> anyhow::Result<Message> {
        let msg = match Command::wade_through("queue", Ok(api
            .queue()
            .await)) {
            Ok(value) => value,
            Err(error) => format!("[ queue[2]: {:?} ]", error)
        };

        bot
            .send_message(message.chat.id, msg)
            .await
            .map_err(|e| anyhow!("[ queue[3]: couldn't answer: {:?} ]", e))
    }

    pub async fn get(bot: AutoSend<Bot>, message: Message, api: Api, title: String) -> anyhow::Result<Message> {
        let msg = match api.
            get_movie_by_title(&title)
            .await {
            Ok(value) => match value {
                None => format!("{} couldn't be found in queue", title),
                Some(movie) => movie.to_string(),
            },
            Err(error) => format!("[ get[2]: {:?} ]", error)
        };

        bot
            .send_message(message.chat.id, msg)
            .await
            .map_err(|e| anyhow!("[ get[3]: couldn't answer: {:?} ]", e))
    }
}

async fn send_poll<S: Into<String>, V: IntoIterator<Item=String>>(question: S, options: V, allow_multiple_answers: bool, is_anonymous: bool, bot: &Bot, chat_id: i64) -> anyhow::Result<Message> {
    Ok(bot
        .send_poll(chat_id, question, options)
        .is_anonymous(is_anonymous)
        .allows_multiple_answers(allow_multiple_answers)
        .send()
        .await?)
}

async fn send_participation_poll(bot: &Bot, chat_id: i64) -> anyhow::Result<Message> {
    let question = "Ich bin";

    let options = env::var("PARTICIPATION_POLL_DEFAULT_OPTIONS")
        .unwrap_or("Dabei,Nicht dabei,Spontan".to_string())
        .split(",")
        .map(|s| s.trim().into())
        .collect::<Vec<String>>();

    send_poll(question, options, false, false, bot, chat_id).await
}

async fn send_movie_poll(api: Api, bot: &Bot, chat_id: i64) -> anyhow::Result<Message> {
    let question = "Which movie do you want to watch?";

    let default_options = env::var("POLL_DEFAULT_OPTIONS")
        .unwrap_or(String::from("Nicht dabei,Mir egal"))
        .split(",")
        .map(|s| s.trim().into())
        .collect::<Vec<String>>();

    let options_count = POLL_MAX_OPTIONS_COUNT - default_options.len();
    let options = match api.queue().await {
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
            let _ = bot.send_message(chat_id, format!("Failed to retrieve movies: {:#?}", err));
            vec![]
        }
    };
    if options.len() == 0 {
        let msg = "poll: no movies in queue (or error decoding individual ones)";
        let _ = bot.send_message(chat_id, msg);
        Err(anyhow!(msg))?
    }

    send_poll(question, options, true, false, bot, chat_id).await
}

async fn answer(
    message: Message,
    bot: AutoSend<Bot>,
    command: Command,
    api: Api,
) -> anyhow::Result<()> {
    log::info!("{:?}", command);

    // TODO: This should be assembled from members of the group at some point (talk to API for that)
    if message.chat.id != env::var("ADMIN_CHAT")?.parse::<i64>()? {
        bot.send_message(message.chat.id, "You're not allowed to use this bot.").await?;
        log::info!("{} ([u]{:?} | [f]{:?} | [l]{:?} | [t]{:?}) is not allowed to use this bot", message.chat.id, message.chat.username(), message.chat.first_name(), message.chat.last_name(), message.chat.title());

        return Ok(());
    }

    match command {
        Command::Help => bot.send_message(message.chat.id, Command::descriptions()).await?,
        Command::WerHatDieHandAnDerMaus => bot.send_message(message.chat.id, format!("Tim")).await?,
        Command::Add(imdb_url) => Command::add_movie(bot, message, api, imdb_url).await?,
        Command::Delete(title) => Command::delete_movie(bot, message, api, title, MovieDeleteStatus::Deleted).await?,
        Command::Queue => Command::queue(bot, message, api).await?,
        Command::Watch(title) => Command::delete_movie(bot, message, api, title, MovieDeleteStatus::Watched).await?,
        Command::Get(title) => Command::get(bot, message, api, title).await?,
        Command::Poll => send_movie_poll(api.clone(), bot.inner(), message.chat.id).await?,
        Command::Noop => bot.send_message(message.chat.id, "").await?,
        command_name => bot.send_message(message.chat.id, format!("{:#?} is not implemented yet.", command_name)).await?,
    };

    Ok(())
}

async fn inline_query_handler(
    query: InlineQuery,
    bot: AutoSend<Bot>,
) -> anyhow::Result<()> {
    Ok(())
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

    if env::args().any(|arg| arg.to_lowercase() == String::from("participation-poll")) {
        println!("{:#?}", send_participation_poll(bot.inner(), env::var("ADMIN_CHAT").expect("`ADMIN_CHAT` has to be set").parse::<i64>()?).await?);

        return Ok(());
    }

    let handler = dptree::entry()
        .branch(Update::filter_message()
            .branch(
                dptree::entry()
                    .filter_command::<Command>()
                    .endpoint(answer)
            )
        )
        .branch(Update::filter_inline_query().endpoint(inline_query_handler));

    let api = Api::new(env::var("BASE_URL").unwrap_or("http://api".to_string()).parse().expect("BASE_URL is in the wrong format"));
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![api])
        .default_handler(|update| async move {
            log::warn!("Unhandler update: {:?}", update)
        })
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occured in the dispatcher",
        ))
        .build()
        .setup_ctrlc_handler()
        .dispatch()
        .await;

    Ok(())
}
