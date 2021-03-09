PRAGMA journal_mode=WAL;

CREATE TABLE rates(
    src TEXT NOT NULL,
    dst TEXT NOT NULL,
    date TEXT, -- ISO datetime
    rate REAL,
    provider TEXT NOT NULL,
    cache_until TEXT NOT NULL, -- ISO datetime
    PRIMARY KEY (src, dst, provider),
    CHECK (src <> dst)
);
CREATE INDEX primary_key_rates ON rates(src, dst, provider);

CREATE TABLE history(
    datetime TEXT, -- ISO datetime
    content TEXT
);
