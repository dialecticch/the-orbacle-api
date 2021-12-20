use self::types::*;

use anyhow::anyhow;
use anyhow::Result;
use futures::StreamExt;
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use reqwest::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use std::num::NonZeroU32;

pub mod types;

static API_BASE: &str = "https://api.opensea.io/api";

static ASSETS_PATH: &str = "/v1/assets/";
static COLLECTION_PATH: &str = "/v1/collection/";

pub struct OpenseaAPIClient {
    rate_limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
    client: reqwest::Client,
}

impl Default for OpenseaAPIClient {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenseaAPIClient {
    pub fn new() -> Self {
        let client = reqwest::Client::new();
        let quota = Quota::per_second(NonZeroU32::new(5u32).unwrap());
        let rate_limiter = RateLimiter::direct(quota);
        Self {
            rate_limiter,
            client: client,
        }
    }

    async fn fetch_page<'a, T: Serialize + ?Sized, L: Serialize + ?Sized, R: ?Sized>(
        &self,
        path: &str,
        query: &T,
        extra_query: &L,
    ) -> Result<R>
    where
        for<'de> R: Deserialize<'de> + 'a,
    {
        self.rate_limiter.until_ready().await;
        let reqw = self
            .client
            .get(API_BASE.to_string() + path)
            .query(&query)
            .query(&extra_query)
            .header("Accept-Encoding", "application/json")
            .build()?;
        //println!("{}", reqw.url());
        let resp = self.client.execute(reqw).await?;
        match resp.status() {
            StatusCode::OK => serde_json::from_str(&resp.text().await?).map_err(|e| e.into()),
            _ => match resp.text().await {
                Ok(text) => Err(anyhow!(text)),
                Err(e) => Err(anyhow!(e)),
            },
        }
    }

    async fn fetch_assets_page(&self, req: AssetsRequest) -> Result<AssetsResponse> {
        self.fetch_page(ASSETS_PATH, &req, &EmptyRequest::default())
            .await
    }

    pub async fn fetch_token_ids(
        &self,
        collection: &str,
        token_ids: Vec<u64>,
    ) -> Result<Vec<Asset>> {
        let req = AssetsRequest::new().collection(collection).build();

        let token_ids = Self::token_ids_query(token_ids);

        let mut results = vec![];
        let mut call = 0;
        while results.len() < token_ids.len() {
            let page: AssetsResponse = self
                .fetch_page(
                    ASSETS_PATH,
                    &req,
                    &token_ids[usize::min(call * 30, token_ids.len())
                        ..usize::min((call + 1) * 30, token_ids.len())],
                )
                .await?;
            call += 1;
            results.extend(page.assets);
        }
        Ok(results)
    }

    async fn get_assets_serial(&self, req: AssetsRequest) -> Result<Vec<Asset>> {
        let mut req = req;
        let wanted = req.limit.unwrap_or(8888);
        req.limit = Some(usize::min(wanted, 50));

        let mut results = vec![];
        while results.len() < wanted {
            let page = self.fetch_assets_page(req.clone()).await?;
            let new_results = page.assets.len();
            if new_results == 0 {
                break;
            }
            if let Some(offset) = req.offset {
                req.offset = Some(offset + new_results);
            } else {
                req.offset = Some(new_results);
            }
            results.extend(page.assets);
        }

        Ok(results)
    }

    async fn get_assets_parallel(&self, req: AssetsRequest) -> Result<Vec<Asset>> {
        let mut req = req;
        let wanted = req.limit.unwrap_or(req.expected.unwrap_or(1000));
        req.limit = Some(usize::min(wanted, 50));

        let mut stream = futures::stream::iter(0..wanted / req.limit.unwrap())
            .map(|i| {
                self.fetch_assets_page(
                    req.clone()
                        .offset(req.offset.unwrap_or(0) + i * req.limit.unwrap())
                        .build(),
                )
            })
            .buffer_unordered(6);

        let mut results = vec![];

        while let Some(result) = stream.next().await {
            match result {
                Ok(mut resp) => {
                    results.append(&mut resp.assets);
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }
        }

        Ok(results)
    }

    pub async fn get_assets(&self, req: AssetsRequest) -> Result<Vec<Asset>> {
        #[allow(unused_variables)]
        if let Some(expected) = req.expected.as_ref() {
            log::info!("Have expected, using parallel fetcher");
            self.get_assets_parallel(req).await
        } else {
            log::info!("Don't know expected, using serial fetcher");
            self.get_assets_serial(req).await
        }
    }

    pub async fn get_single_asset(
        &self,
        collection_slug: &str,
        token_id: Vec<u64>,
    ) -> Result<Asset> {
        let req = AssetsRequest::new()
            .collection(collection_slug)
            .token_ids(token_id)
            .build();
        match self.fetch_assets_page(req).await {
            Ok(r) => Ok(r.assets[0].clone()),
            Err(e) => Err(anyhow!(e)),
        }
    }

    pub async fn get_collection(&self, collection_slug: &str) -> Result<CollectionResponse> {
        let reqw = self
            .client
            .get(API_BASE.to_string() + COLLECTION_PATH + collection_slug)
            .header("Accept-Encoding", "application/json")
            .build()?;
        let resp = self.client.execute(reqw).await?;
        match resp.status() {
            StatusCode::OK => serde_json::from_str(&resp.text().await?).map_err(|e| e.into()),
            _ => match resp.text().await {
                Ok(text) => Err(anyhow!(text)),
                Err(e) => Err(anyhow!(e)),
            },
        }
    }

    fn token_ids_query(token_ids: Vec<u64>) -> Vec<(String, String)> {
        let mut b = vec![];
        for i in token_ids {
            b.push(("token_ids".to_string(), i.to_string()));
        }
        b
    }
}
