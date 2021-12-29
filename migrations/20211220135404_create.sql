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
    address VARCHAR NOT NULL,
    total_supply INT NOT NULL,
    rarity_cutoff float not NULL,
    floor_price float not null,
    primary key (slug)
);

CREATE TABLE TRAIT (
    collection_slug VARCHAR NOT NULL,
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
    token_id int NOT NULL,
    price float,
    timestamp INT NOT NULL,

    primary key (collection_slug, token_id, timestamp)
);