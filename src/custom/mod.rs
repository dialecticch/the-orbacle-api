use anyhow::Result;
use std::collections::HashMap;
use std::fs::File;

pub fn read_custom_price(collection_slug: &str, token_id: i32) -> Result<Option<f64>> {
    match dotenv::var("CUSTOM_PRICES_JSON_PATH") {
        Ok(path) => {
            let map: HashMap<String, HashMap<String, f64>> =
                serde_json::from_reader(File::open(path)?)?;

            match map.get(collection_slug) {
                Some(c) => match c.get(&token_id.to_string()) {
                    Some(p) => Ok(Some(*p)),
                    None => Ok(None),
                },
                None => Ok(None),
            }
        }
        Err(_) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_read() {
        assert_eq!(read_custom_price("0xmons-xyz", 249).unwrap(), Some(20.0))
    }
}
