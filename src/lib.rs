use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

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
            status => unimplemented!("status is unimplemented: {}", status)
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Queue {
    queue: QueueMovie,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QueueMovie {
    id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Movie {
    id: String,
    #[serde(rename = "imdbData")]
    imdb_data: ImdbMovie,
    status: MovieStatus,
}

impl fmt::Display for Movie {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}) {}⭐", self.imdb_data.title, self.imdb_data.year, self.imdb_data.rating)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImdbMovie {
    #[serde(rename = "imdbId")]
    imdb_id: String,
    title: String,
    year: u32,
    rating: f32,
}
