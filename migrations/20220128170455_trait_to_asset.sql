ALTER TABLE Trait
ADD COLUMN token_ids int[] NOT NULL DEFAULT array[]::int[];