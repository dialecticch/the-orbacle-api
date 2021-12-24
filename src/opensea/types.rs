use chrono::NaiveDateTime;
use serde_aux::prelude::*;

use std::collections::HashMap;

#[derive(Default, serde::Serialize)]
pub struct EmptyRequest {}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CollectionResponse {
    pub collection: Collection,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Collection {
    pub primary_asset_contracts: Vec<AssetContract>,
    pub traits: HashMap<String, HashMap<String, u64>>,
    pub slug: String,
    pub stats: CollectionStats,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CollectionStats {
    pub total_supply: f64,
    pub total_sales: f64,
    pub floor_price: f64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct SellOrder {
    pub sale_kind: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub current_price: f64,
    pub payment_token: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Owner {
    pub address: String,
}
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Trait {
    trait_type: String,
    #[serde(deserialize_with = "deserialize_string_from_number")]
    pub value: String,
    display_type: Option<String>,
    max_value: Option<u64>,
    pub trait_count: Option<u64>,
    order: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum AssetContractType {
    #[serde(rename = "fungible")]
    Fungible,
    #[serde(rename = "non-fungible")]
    NonFungible,
    #[serde(rename = "semi-fungible")]
    SemiFungible,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum SchemaName {
    ERC721,
    ERC1155,
    CRYPTOPUNKS,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AssetContract {
    pub address: String,
    asset_contract_type: AssetContractType,
    created_date: NaiveDateTime,
    name: Option<String>,
    nft_version: Option<String>,
    opensea_version: Option<String>,
    owner: Option<u64>,
    schema_name: SchemaName,
    symbol: Option<String>,
    #[serde(deserialize_with = "deserialize_option_number_from_string")]
    pub total_supply: Option<u64>,
    description: Option<String>,
    external_link: Option<String>,
    image_url: Option<String>,
    default_to_fiat: bool,
    dev_buyer_fee_basis_points: u64,
    dev_seller_fee_basis_points: u64,
    only_proxied_transfers: bool,
    opensea_buyer_fee_basis_points: u64,
    opensea_seller_fee_basis_points: u64,
    buyer_fee_basis_points: u64,
    seller_fee_basis_points: u64,
    payout_address: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EmbeddedAsset {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub token_id: u64,
    num_sales: u64,
    pub name: Option<String>,
    description: Option<String>,
    image_preview_url: Option<String>,
    pub permalink: Option<String>,
    decimals: Option<u64>,
    asset_contract: Option<AssetContract>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Asset {
    pub name: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub token_id: u64,
    pub image_url: String,
    pub sell_orders: Option<Vec<SellOrder>>,
    pub traits: Option<Vec<Trait>>,
    pub owner: Owner,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AssetsResponse {
    pub assets: Vec<Asset>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AssetStub {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub token_id: u64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PaymentToken {
    id: u64,
    pub symbol: String,
    address: String,
    image_url: String,
    pub name: String,
    pub decimals: u32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub eth_price: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub usd_price: f64,
}

impl PaymentToken {
    pub fn display_amount(&self, input_amount: u64) -> f64 {
        let div = 10u64.pow(self.decimals - 2);
        (input_amount / div) as f64 / 100.0
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderBy {
    TokenId,
    SaleDate,
    SaleCount,
    SalePrice,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AssetsRequest {
    owner: Option<String>,
    pub token_ids: Option<Vec<u64>>,
    asset_contract_address: Option<String>,
    asset_contract_addresses: Option<Vec<String>>,
    order_by: Option<OrderBy>,
    order_direction: Option<OrderDirection>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    collection: Option<String>,
    #[serde(skip_serializing)]
    pub expected: Option<usize>,
}

impl AssetsRequest {
    pub fn new() -> Self {
        Self {
            owner: None,
            token_ids: None,
            asset_contract_address: None,
            asset_contract_addresses: None,
            order_by: None,
            order_direction: None,
            offset: None,
            limit: None,
            collection: None,
            expected: None,
        }
    }

    pub fn owner(&mut self, arg: &str) -> &mut Self {
        self.owner = Some(arg.to_string());
        self
    }

    pub fn token_ids(&mut self, arg: Vec<u64>) -> &mut Self {
        self.token_ids = Some(arg);
        self
    }

    pub fn asset_contract_address(&mut self, arg: &str) -> &mut Self {
        self.asset_contract_address = Some(arg.to_string());
        self
    }

    pub fn limit(&mut self, arg: usize) -> &mut Self {
        self.limit = Some(arg);
        self
    }

    pub fn offset(&mut self, arg: usize) -> &mut Self {
        self.offset = Some(arg);
        self
    }

    pub fn collection(&mut self, arg: &str) -> &mut Self {
        self.collection = Some(arg.to_string());
        self
    }

    pub fn expected(&mut self, arg: usize) -> &mut Self {
        self.expected = Some(arg);
        self
    }

    pub fn build(&mut self) -> Self {
        self.clone()
    }
}

impl Default for AssetsRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub asset: Option<EmbeddedAsset>,
    pub total_price: Option<String>,
    pub ending_price: Option<String>,
    pub created_date: NaiveDateTime,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EventsResponse {
    pub asset_events: Vec<Event>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EventsRequest {
    asset_contract_address: Option<String>,
    collection_slug: Option<String>,
    token_id: Option<u64>,
    account_address: Option<String>,
    event_type: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,

    occurred_after: Option<String>,
    occurred_before: Option<String>,
    #[serde(skip_serializing)]
    pub expected: Option<usize>,
}
impl Default for EventsRequest {
    fn default() -> Self {
        Self::new()
    }
}
impl EventsRequest {
    pub fn new() -> Self {
        Self {
            asset_contract_address: None,
            collection_slug: None,
            token_id: None,
            account_address: None,
            event_type: None,
            occurred_after: None,
            occurred_before: None,
            limit: None,
            offset: None,
            expected: None,
        }
    }

    pub fn event_type(&mut self, arg: &str) -> &mut Self {
        self.event_type = Some(arg.to_string());
        self
    }

    pub fn asset_contract_address(&mut self, arg: &str) -> &mut Self {
        self.asset_contract_address = Some(arg.to_string());
        self
    }

    pub fn occurred_after(&mut self, arg: &str) -> &mut Self {
        self.occurred_after = Some(arg.to_string());
        self
    }

    pub fn occurred_before(&mut self, arg: &str) -> &mut Self {
        self.occurred_before = Some(arg.to_string());
        self
    }

    pub fn expected(&mut self, arg: usize) -> &mut Self {
        self.expected = Some(arg);
        self
    }

    pub fn token_id(&mut self, arg: u64) -> &mut Self {
        self.token_id = Some(arg);
        self
    }

    pub fn limit(&mut self, arg: usize) -> &mut Self {
        self.limit = Some(arg);
        self
    }

    pub fn offset(&mut self, arg: usize) -> &mut Self {
        self.offset = Some(arg);
        self
    }

    pub fn build(&mut self) -> Self {
        self.clone()
    }
}
