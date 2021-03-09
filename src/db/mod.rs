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

//! Module grouping all db related concern

use std::convert::TryInto;

use anyhow::Result;
use chrono::Local;
use log::{debug, trace, warn};
use rusqlite::named_params;
use rusqlite::Connection;
use serde_rusqlite::columns_from_statement;
use serde_rusqlite::from_row_with_columns;
use serde_rusqlite::to_params_named;

mod migrations;
mod rate;

use migrations::MIGRATIONS;
use rate::RateInternal;

use crate::config::Config;
use crate::currency::Currency;
use crate::rate::Rate;

#[cfg(test)]
mod tests;

/// Store and bucket, represent the whole database
pub struct Db {
    conn: Connection,
}

impl Db {
    /// Initialize the rate database
    pub fn new(cfg: &Config) -> Result<Self> {
        trace!("Initialize database");
        let conn = Connection::open(cfg.db_path())?;
        Db::init(conn)
    }

    /// Initialize database, in particular, apply migrations for the schema
    fn init(mut conn: Connection) -> Result<Self> {
        MIGRATIONS.latest(&mut conn)?;
        dbg!(conn.is_autocommit());
        Ok(Db { conn })
    }

    /// In memory database, mainly for testing
    #[cfg(test)]
    fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Db::init(conn)
    }
}

impl Db {
    /// Retrieve rates from a currency to another. First member of the tuple
    /// contains up-to-date rates and the second member outdated ones
    pub fn get_rates<'c>(
        &self,
        src: &'c Currency,
        dst: &'c Currency,
        provider: &str,
    ) -> Result<(Vec<Rate<'c>>, Vec<Rate<'c>>)> {
        trace!("get_rates({}, {}, {:?})", src, dst, provider);
        // Hard code this to limit storage overhead
        if src == dst {
            warn!("Same source and destination currency, don’t store");
            return Ok((vec![Rate::parity(src)], vec![]));
        }

        let mut stmt = self.conn.prepare(
            "SELECT * FROM rates \
             WHERE src = :src AND dst = :dst AND provider = :provider",
        )?;
        let columns = columns_from_statement(&stmt);
        let mut rows = stmt.query_named(named_params!{
        ":src": src.get_main_iso(),
        ":dst": dst.get_main_iso(),
        ":provider": provider,
        })?;

        let now = Local::now();
        let (mut uptodate_rates, mut outdated_rates): (Vec<Rate>, Vec<Rate>) =
            (Vec::new(), Vec::new());
        while let Some(row) = rows.next()? {
            let rate_internal = from_row_with_columns::<RateInternal>(row, &columns)?;
            let rate: Rate = rate_internal.try_into()?;
            match rate.cache_until() {
                Some(date) if date > &now => uptodate_rates.push(rate),
                _ => outdated_rates.push(rate),
            }
        }
        trace!("uptodate_rates: {:?}", uptodate_rates);
        trace!("outdated_rates: {:?}", outdated_rates);

        Ok((uptodate_rates, outdated_rates))
    }

    /// Set rate from a currency to another
    pub fn set_rate(&self, rate: &Rate) -> Result<()> {
        if rate.src() == rate.dst() {
            warn!("Same source and destination currency, don’t store");
            return Ok(());
        }
        let ri: RateInternal = rate.try_into()?;
        let n = self.conn.execute_named(
            "INSERT OR REPLACE INTO rates (src, dst, date, rate, provider, cache_until) VALUES (:src, :dst, :date, :rate, :provider, :cache_until)",
            &to_params_named(ri)?.to_slice(),
        )?;
        debug!("Upserted {} rows.", n);
        Ok(())
    }

    /// Remove rate from a currency to another
    pub fn del_rate(&self, rate: &Rate) -> Result<()> {
        trace!("Remove rate {:?} from databse.", rate);
        let rate_internal: RateInternal = rate.try_into()?;
        let n = self.conn.execute_named(
            "DELETE FROM TABLE rates \
             WHERE src = :src AND dst = :dst",
            &to_params_named(rate_internal)?.to_slice(),
        )?;
        debug!("Changed {} rows.", n);
        Ok(())
    }
}
