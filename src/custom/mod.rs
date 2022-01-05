use anyhow::Result;
use std::collections::HashMap;
use std::fs::File;

pub fn read_custom_price(collection_slug: &str, token_id: i32) -> Result<Option<f64>> {
    let map: HashMap<String, HashMap<String, f64>> =
        serde_json::from_reader(File::open("./src/custom/custom_prices.json")?)?;

    match map.get(collection_slug) {
        Some(c) => match c.get(&token_id.to_string()) {
            Some(p) => return Ok(Some(*p)),
            None => return Ok(None),
        },
        None => return Ok(None),
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
