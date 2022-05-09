use std::fmt::{Debug, Formatter};
use anyhow::anyhow;
use reqwest::{Client, StatusCode};
use reqwest::Response;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use teloxide_core::net;
use thiserror::Error;
use url::Url;

use crate::{Movie, MovieDeleteStatus, Movies, Queue};

pub enum ResolverApiEndpoints {
    Search,
}

#[derive(Serialize)]
struct SearchTitleRequest {
    title: String,
    year: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct SearchItem {
    pub title: String,
    pub id: Option<String>,
    #[serde(rename(deserialize = "type"))]
    pub _type: Option<String>,
    pub year: Option<u16>,
}

impl ToString for SearchItem {
    fn to_string(&self) -> String {
        let mut s = self.title.clone();
        s + &self.year.map(|s| format!(" ({})", s.to_string())).unwrap_or("".to_string())
    }
}

impl ToString for ResolverApiEndpoints {
    fn to_string(&self) -> String {
        match self {
            ResolverApiEndpoints::Search => "search"
        }.to_string()
    }
}

#[derive(Debug, Deserialize)]
pub struct StreamingproviderResolverResponseItem {
    pub name: String,
    pub movies: Vec<SearchItem>,
}

impl ToString for StreamingproviderResolverResponseItem {
    fn to_string(&self) -> String {
        let mut msg = self.name.clone() + ":\n";
        msg += &*(String::from("  ") + &*self.movies
            .iter()
            .map(|si| si.to_string())
            .collect::<Vec<String>>()
            .join("\n  "));

        msg
    }
}

#[derive(Debug, Deserialize)]
pub struct StreamingproviderResolverResponse {
    pub results: Vec<StreamingproviderResolverResponseItem>,
}

#[derive(Clone)]
pub struct ResolverApi {
    client: Client,
    base_url: Url,
}

impl ResolverApi {
    pub fn new(base_url: Url) -> ResolverApi {
        ResolverApi {
            client: net::default_reqwest_settings()
                .build()
                .expect("Client creation failed"),
            base_url,
        }
    }

    fn join_on_base_url(&self, endpoint: &String) -> anyhow::Result<Url> {
        Ok(self.base_url.join(endpoint.as_str())?)
    }

    pub async fn search(&self, title: String) -> anyhow::Result<anyhow::Result<StreamingproviderResolverResponse>> {
        self.post(ResolverApiEndpoints::Search.to_string(), SearchTitleRequest { title, year: None })
            .await
            .map(|body| ResolverApi::decode_body::<StreamingproviderResolverResponse>(body))
            .map_err(|e| anyhow!("[ ResolverApi::search[2]: failed to `GET` endpoint: {:?} ]", e))
    }

    fn decode_body<'a, R: DeserializeOwned>(response: Response) -> anyhow::Result<R> {
        log::info!("decode_body");
        let text = async_std::task::block_on(response.text())?;
        log::info!("decode_body:text:{}", text);
        serde_json::from_str(&text)
            .map_err(|e| anyhow!("[ decode_body: unable to decode response body for: {}, [ {:?} ] ]", std::any::type_name::<R>(), e))
    }

    async fn post<T: Serialize>(&self, path: String, body: T) -> anyhow::Result<Response> {
        let url = self.join_on_base_url(&path)?;

        Ok(self
            .client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("[ post[0]: failed sending: {:?} ]", e))?)
    }
}
