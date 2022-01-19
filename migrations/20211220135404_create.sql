CREATE TABLE ASSET (
    name VARCHAR NOT NULL,
    collection_slug VARCHAR NOT NULL,
    token_id INT NOT NULL,
    image_url VARCHAR NOT NULL,
    owner VARCHAR NOT NULL,
    traits VARCHAR[] NOT NULL,
    unique_traits INT NOT NULL,
    traits_3_combination_overlap INT NOT NULL,
    traits_4_combination_overlap INT NOT NULL,
    traits_5_combination_overlap INT NOT NULL,
    traits_3_combination_overlap_ids INT[] NOT NULL,
    traits_4_combination_overlap_ids INT[] NOT NULL,
    traits_5_combination_overlap_ids INT[] NOT NULL,
    primary key (collection_slug, token_id)
);

CREATE TABLE COLLECTION (
    slug VARCHAR NOT NULL,
    name VARCHAR NOT NULL,
    address VARCHAR NOT NULL,
    banner_image_url VARCHAR NOT NULL,
    total_supply INT NOT NULL,
    rarity_cutoff float not NULL,
    floor_price float not null,
    daily_volume float not null,
    daily_sales float not null,
    daily_avg_price float not null,
    weekly_avg_price float not null,
    monthly_avg_price float not null,
    nr_owners float not null,
    avg_trait_rarity float not null,
    primary key (slug)
);


CREATE TABLE TRAIT (
    collection_slug VARCHAR NOT NULL,
    trait_id VARCHAR NOT NULL,
    trait_type VARCHAR NOT NULL,
    trait_name VARCHAR NOT NULL,
    trait_count INT NOT NULL,

    primary key (collection_slug, trait_type, trait_name)
);

CREATE TABLE SALE (
    collection_slug VARCHAR NOT NULL,
    token_id int NOT NULL,
    price float NOT NULL,
    timestamp INT NOT NULL,

    primary key (collection_slug, token_id, timestamp)
);


CREATE TABLE LISTING (
    collection_slug VARCHAR NOT NULL,
    update_type VARCHAR NOT NULL,
    token_id int NOT NULL,
    price float,
    timestamp INT NOT NULL,

    primary key (collection_slug, token_id, timestamp)
);


CREATE INDEX collections_ordered ON collection(slug DESC);
CREATE INDEX assets_ordered ON asset(collection_slug DESC, token_id DESC);
CREATE INDEX traits_ordered ON trait(collection_slug DESC, trait_id DESC);
CREATE INDEX sales_ordered ON sale(collection_slug DESC, timestamp DESC);
CREATE INDEX listings_ordered ON listing(collection_slug DESC, timestamp DESC);