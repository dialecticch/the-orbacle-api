pub mod liquidty;
pub mod listings;
pub mod prices;
pub mod rarities;
pub mod sales;
pub mod wallet;

use chrono::NaiveDateTime;
#[derive(Default, Clone, Debug)]
pub struct TokenListing {
    pub token_id: i32,
    pub price: Option<f64>,
}
#[derive(Default, Clone, Debug, PartialEq)]
pub struct TraitListing {
    pub token_id: i32,
    pub price: f64,
}

#[derive(Default, Clone, Debug)]
pub struct RarestTraitFloor {
    pub token_id: i32,
    pub trait_id: String,
    pub floor_price: f64,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct TraitFloor {
    pub trait_id: String,
    pub token_id: i32,
    pub floor_price: f64,
}

#[derive(Default, Clone, Debug)]
pub struct TraitRarities {
    pub trait_id: String,
    pub rarity: f64,
}

#[derive(Clone, Debug)]
pub struct TokenSale {
    pub token_id: i32,
    pub time: NaiveDateTime,
    pub price: f64,
}

impl Default for TokenSale {
    fn default() -> Self {
        Self {
            token_id: 0i32,
            time: NaiveDateTime::from_timestamp(0, 0),
            price: 0f64,
        }
    }
}
