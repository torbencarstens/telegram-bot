use anyhow::anyhow;
use reqwest::Client;
use reqwest::Response;
use serde::de::DeserializeOwned;
use serde::Serialize;
use teloxide_core::net;
use thiserror::Error;
use url::Url;

use crate::Movie;

pub enum ApiEndpoints {
    AddMovie,
    DeleteMovie(String)
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

impl ToString for ApiEndpoints {
    fn to_string(&self) -> String {
        match self {
            ApiEndpoints::AddMovie => "movie".to_string(),
            ApiEndpoints::DeleteMovie(id) => {
                format!("movie/{}", id)
            }
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

    fn join_on_base_url(&self, endpoint: String) -> anyhow::Result<Url> {
        Ok(self.base_url.join(endpoint.as_str())?)
    }

    pub async fn add_movie(&self, imdb_url: Url) -> anyhow::Result<anyhow::Result<anyhow::Result<Movie>>> {
        Ok(self.put(ApiEndpoints::AddMovie.to_string(), AddMovieRequest { url: imdb_url })
            .await
            .map(|body| {
                async { Api::decode_body::<Movie>(body).await }
            })
            .map_err(|e| anyhow!("[ Api::add_movie[2]: failed to decode body: {:?} ]", e))?
            .await)
    }

    pub async fn delete_movie(&self, id: String) -> anyhow::Result<anyhow::Result<anyhow::Result<Movie>>> {
        Ok(self.delete(ApiEndpoints::DeleteMovie(id.clone()).to_string())
            .await
            .map(|body| {
                async { Api::decode_body(body).await }
            })
            .map_err(|e| anyhow!("[ Api::delete_movie[0]: failed to decode body: {:?} ]", e))?
            .await)
    }

    async fn decode_body<'a, R: DeserializeOwned>(response: Response) -> anyhow::Result<anyhow::Result<R>> {
        let response_value = response
            .json::<R>()
            .await;
        Ok(response_value
            .map_err(|e| anyhow!("[ decode_body: unable to decode response body for: {}, [ {:?} ] ]", std::any::type_name::<R>(), e)))
    }

    async fn put<T: Serialize>(&self, path: String, body: T) -> anyhow::Result<Response> {
        let url = self.join_on_base_url(path)?;

        Ok(self
            .client
            .put(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("[ post[0]: failed sending: {:?} ]", e))?)
    }

    async fn delete(&self, path: String) -> anyhow::Result<Response> {
        let url = self.join_on_base_url(path)?;

        Ok(self
            .client
            .delete(url)
            .send()
            .await
            .map_err(|error| anyhow!("[ delete[0]: failed sending: {:?} ]", error))?)
    }
}
