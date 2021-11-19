use anyhow::anyhow;
use reqwest::{Client, StatusCode};
use reqwest::Response;
use serde::de::DeserializeOwned;
use serde::Serialize;
use teloxide_core::net;
use thiserror::Error;
use url::Url;

use crate::{Movie, MovieDeleteStatus, Movies, MovieStatus, Queue};

pub enum ApiEndpoints<'a> {
    AddMovie,
    DeleteMovie(&'a String, MovieDeleteStatus),
    GetMovie(&'a String),
    Queue,
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Generic")]
    Generic(String)
}

#[derive(Serialize)]
struct AddMovieRequest {
    #[serde(rename(serialize = "imdbUrl"))]
    url: Url,
}

impl<'a> ToString for ApiEndpoints<'a> {
    fn to_string(&self) -> String {
        match self {
            ApiEndpoints::AddMovie => "movie".to_string(),
            ApiEndpoints::DeleteMovie(id, status) => {
                format!("queue/{}?status={}", id, status.to_string())
            },
            ApiEndpoints::GetMovie(id) => {
                format!("movie/{}", id)
            }
            ApiEndpoints::Queue => "queue".to_string(),
        }.to_string()
    }
}

#[derive(Clone)]
pub struct Api {
    client: Client,
    base_url: Url,
}

impl Api {
    pub fn new(base_url: Url) -> Api {
        Api {
            client: net::default_reqwest_settings()
                .build()
                .expect("Client creation failed"),
            base_url,
        }
    }

    fn join_on_base_url(&self, endpoint: &String) -> anyhow::Result<Url> {
        Ok(self.base_url.join(endpoint.as_str())?)
    }

    pub async fn add_movie(&self, imdb_url: Url) -> anyhow::Result<anyhow::Result<Movie>> {
        self.put(ApiEndpoints::AddMovie.to_string(), AddMovieRequest { url: imdb_url })
            .await
            .and_then(|response| if response.status() == StatusCode::CONFLICT { Err(anyhow!("movie already exists")) } else { Ok(response) })
            .map(|body| Api::decode_body::<Movie>(body))
            .map_err(|e| anyhow!("[ Api::add_movie[2]: failed to `PUT` to endpoint: {:?} ]", e))
    }

    pub async fn delete_movie(&self, id: String, status: MovieDeleteStatus) -> anyhow::Result<anyhow::Result<Movie>> {
        self.delete(ApiEndpoints::DeleteMovie(&id, status).to_string())
            .await
            .map(|body| {
                println!("{:?}", body.status());

                Api::decode_body(body)
            })
            .map_err(|e| anyhow!("[ Api::delete_movie[0]: failed to delete movie from endpoint: {:?} ]", e))
    }

    pub async fn get_movie(&self, id: &String) -> anyhow::Result<anyhow::Result<Movie>> {
        self.get(ApiEndpoints::GetMovie(id).to_string())
            .await
            .map(Api::decode_body)
            .map_err(|e| anyhow!("[  Api::get_movie[0]: failed to retrieve movie: {:?} ]", e))
    }

    pub async fn queue(&self) -> anyhow::Result<Movies> {
        let mut movies: Vec<anyhow::Result<anyhow::Result<Movie>>> = vec![];
        match self.get(ApiEndpoints::Queue.to_string())
            .await
        {
            Ok(response) => {
                let queue: Queue = Api::decode_body(response)?;
                for queue_movie in queue.queue {
                    let movie = self.get_movie(&queue_movie.id);
                    movies.push(movie.await);
                }
                Ok(())
            }
            Err(e) => Err(anyhow!("queue:failed retrieving queue:{:?}", e)),
        }?;

        Ok(Movies { movies: movies.into_iter().collect::<anyhow::Result<Vec<anyhow::Result<Movie>>>>()? })
    }

    fn decode_body<'a, R: DeserializeOwned>(response: Response) -> anyhow::Result<R> {
        log::info!("decode_body");
        let text = async_std::task::block_on(response.text())?;
        log::info!("decode_body:text:{}", text);
        serde_json::from_str(&text)
            .map_err(|e| anyhow!("[ decode_body: unable to decode response body for: {}, [ {:?} ] ]", std::any::type_name::<R>(), e))
    }

    async fn get(&self, path: String) -> anyhow::Result<Response> {
        println!("Api::get:{}{}", self.base_url, path);

        let url = self.join_on_base_url(&path)?;
        println!("Api::get:url:{}", url);

        let res = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow!("[  get[0]: failed getting {}: {:?} ]", path, e));

        log::info!("decode_body:res:{:?}", &res);

        res
    }

    async fn put<T: Serialize>(&self, path: String, body: T) -> anyhow::Result<Response> {
        let url = self.join_on_base_url(&path)?;

        Ok(self
            .client
            .put(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("[ post[0]: failed sending: {:?} ]", e))?)
    }

    async fn delete(&self, path: String) -> anyhow::Result<Response> {
        let url = self.join_on_base_url(&path)?;

        Ok(self
            .client
            .delete(url)
            .send()
            .await
            .map_err(|error| anyhow!("[ delete[0]: failed sending: {:?} ]", error))?)
    }
}
