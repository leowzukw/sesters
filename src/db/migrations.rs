/*
Sesters: easily convert one currency to another
Copyright (C) 2018-2019  Clément Joly <oss+sesters@131719.xyz>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

//! Migrations for the database

use lazy_static::lazy_static;
use rusqlite_migration::{Migrations, M};

lazy_static! {
    pub static ref MIGRATIONS: Migrations<'static> = Migrations::new(vec![
        M::up(
            r#"
PRAGMA journal_mode=WAL;
        "#
        ),
        M::up(
            r#"
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
        "#
        ),
        M::up(
            r#"
CREATE INDEX primary_key_rates ON rates(src, dst, provider);
        "#
        ),
        M::up(
            r#"
CREATE TABLE history(
    datetime TEXT, -- ISO datetime
    content TEXT
);
            "#
        )
    ]);
}
