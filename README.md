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
