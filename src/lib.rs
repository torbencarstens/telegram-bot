use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

pub mod api;

const QUEUED_STR: &str = "Queued";
const WATCHED_STR: &str = "Watched";
const DELETED_STR: &str = "Deleted";

#[derive(Debug, Deserialize, Serialize)]
pub enum MovieStatus {
    Queued,
    Watched,
    Deleted,
}

impl ToString for MovieStatus {
    fn to_string(&self) -> String {
        match self {
            MovieStatus::Queued => QUEUED_STR,
            MovieStatus::Watched => WATCHED_STR,
            MovieStatus::Deleted => DELETED_STR
        }.to_string()
    }
}

impl FromStr for MovieStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            QUEUED_STR => Ok(MovieStatus::Queued),
            WATCHED_STR => Ok(MovieStatus::Watched),
            DELETED_STR => Ok(MovieStatus::Deleted),
            // TODO: throw error
            status => Err(anyhow!("{} is not implemented for `MovieStatus`", status))
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Queue {
    queue: Vec<QueueMovie>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QueueMovie {
    id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Movie {
    id: String,
    imdb: ImdbMovie,
    status: MovieStatus,
}

#[derive(Debug)]
pub struct Movies {
    pub movies: Vec<anyhow::Result<Movie>>
}

impl fmt::Display for Movies {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self
            .movies
            .iter()
            .map(|movie| match movie {
                Ok(movie) => format!("{}", movie),
                Err(e) => format!("failed retrieving movie: {:?}", e)
            })
            .fold(String::new(), |acc, s| acc + s.as_str() + "\n"))
    }
}

impl fmt::Display for Movie {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}) {}⭐", self.imdb.title, self.imdb.year, self.imdb.rating)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImdbCover {
    url: String,
    ratio: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImdbMovie {
    id: String,
    title: String,
    year: u32,
    rating: String,
    cover: ImdbCover,
}
