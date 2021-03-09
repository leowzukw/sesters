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

// Store and retrieve user configuration

use clap::crate_name;
use log::info;
use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

fn data_dir() -> PathBuf {
    let mut path = dirs_next::data_dir().unwrap();
    path.push(crate_name!());
    path
}

/// Configuration file
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// Version of the config file
    version: u8,
    /// Currencies to convert to
    currencies: Vec<String>,
    /// Path of the database (directory). Please note that ~ is not expanded
    db_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let mut db_path = dbg!(data_dir());
        db_path.push("db.sqlite3");
        Config {
            version: 0,
            currencies: vec!["EUR".to_string(), "USD".to_string(), "GBP".to_string()],
            db_path,
        }
    }
}

impl Config {
    /// Get current configuration
    pub fn new() -> Result<Config, confy::ConfyError> {
        info!("Reading configuration");
        confy::load(crate_name!())
    }

    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }

    pub fn currencies(&self) -> &Vec<String> {
        &self.currencies
    }
}
