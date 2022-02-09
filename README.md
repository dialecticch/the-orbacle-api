# The Orbacle: A Mutli-Dimensional Approach to NFT Pricing

Frontend Repo: https://github.com/dialecticch/the-orbacle-app

## Introduction

This repository contains the codebase for the Orbacle API. The system is composed of a PostgreSQL database, logic to fetch static and dynamic data from OpenSea and a REST API to query the data in a multitude of ways.

The structure is such:

- `analyzers/`: contains all data processing logic, these methods are called when generating the price/liquidity profiles, data is fetched from the database and analyzed according to the strategies
- `api/`: contains the REST API and server, we use the `rweb` rust crate for this
- `custom/`: contains logic to apply custom prices, this is enabled by setting the `CUSTOM_PRICES_JSON_PATH` env variable
- `opensea/`: contains all the logic to fetch data from the OpenSea API, this is a high level implementation of the Opensea client
- `profiles/`: contains the structures for the different profiles (price, liquidity, rarity, ecc). The Profilesa re constructed here with data from the analyzers and are passed back to the API for response
- `storage/`: contains the database models as well as all direct queries to read/write from postgres, we use the `sqlx` rust crate for this
- `sync/`: contains logic to sync the state of the database with Opensea, this logic runs in a separate thread and is executed every 30min

Upon adding a new collection all static data is fetched from opensea and preprocessed, this includes populating the asset and trait tables, computing trait overlaps, rarity related metrics and getting a first list of listings at that point in time.  
Subsequently the listings, sales and transfers ecc. are synced every 30min. When API requests come in no requests are made to OpenSea, instead date already stored in the DB is used to generate the profiles.  
This has a benefit on loading times as all data is already available locally, but may cause some discrepancy against recently events.

## API Docs

Full REST documentation is available at: https://api.prod.theorbacle.com/docs

## Contributions

Contributions are much appreciated!
Anything from performance improvements, complexity reduction, new strategies or new profiles, if you want to build it please do!

## Development and Build

To run the Orbacle for yourself first create a `.env` file with the following parameters:

```env
DATABASE_URL =
OPENSEA_API_KEY =
ENDPOINT =
PORT =
ADMIN_API_KEY =
CUSTOM_PRICES_JSON_PATH =
```

All values except for `CUSTOM_PRICES_JSON_PATH` are required, if the custom price file is not set, no custom prices will be applied, if it is set, custom prices will be read from a JSON file which needs to have the following format:

```json
{
  "collection-slug": {
    "token-id-1": price,
    "token-id-2": price
  },
  "forgottenruneswizardscult": {
    "777": 1000.0
  }
}
```

Once this is set apply the migrations to the database:

```shell
cargo sqlx migrate run
```

Then you can start the server by running:

```shell
cargo run --release --bin the-orbacle
```

## Calling the Admin API endpoints

To add collections to the Orbacle you can call the `/admin/collection` POST endpoint with the API key set in the `ADMIN_API_KEY` env set in `x-api-key` header:

```curl
curl --location --request POST '<endpoint>:<port>/admin/collection' \
--header 'x-api-key: <admin-api-key>' \
--header 'Content-Type: application/json' \
--data-raw '{
    "collection_slug":"forgottenruneswizardscult",
    "total_supply_expected": 10000,
    "rarity_cutoff_multiplier": 2,
    "ignored_trait_types_rarity": [
    ],
    "ignored_trait_types_overlap": [
        "background"
    ]
}'
```
