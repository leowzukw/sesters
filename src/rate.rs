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

//! Store common representation for rates

use chrono::offset::Local as LocalTime;
use chrono::prelude::*;
use chrono::Duration;

use std::fmt;

use crate::currency::{Currency, USD};

#[cfg(test)]
mod tests {}

/// Rate from a source currency to a destination currency
#[derive(Clone, PartialOrd, PartialEq, Debug)]
pub struct Rate<'c> {
    /// Source currency
    src: &'c Currency,
    /// Destination currency
    dst: &'c Currency,
    /// Date and time the rate was obtained
    date: DateTime<LocalTime>,
    /// Exchange rate
    rate: f64,
    /// Service which provided the rate
    provider: String,
    /// Cache until this date. If None, can’t be cached
    cache_until: Option<DateTime<LocalTime>>,
}

impl<'c> Default for Rate<'c> {
    fn default() -> Self {
        Rate {
            src: &USD,
            dst: &USD,
            date: Local::now(),
            rate: 0.,
            provider: String::from("DEFAULT"),
            cache_until: None,
        }
    }
}

impl<'c> fmt::Display for Rate<'c> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "1 {src} ≈ {rate:.*} {dst} ({date} - {provider})",
            3,
            rate = self.rate(),
            src = self.src(),
            dst = self.dst(),
            date = self.date().format("%F %T"),
            provider = self.provider()
        )
    }
}

impl<'c> Rate<'c> {
    /// Instanciate
    pub fn new(
        src: &'c Currency,
        dst: &'c Currency,
        date: DateTime<LocalTime>,
        rate: f64,
        provider: String,
        cache_until: Option<DateTime<LocalTime>>,
    ) -> Self {
        Rate {
            src,
            dst,
            date,
            rate,
            provider,
            cache_until,
        }
    }

    /// New rate with date set to now (local time), with an optional caching duration from now
    pub fn now(
        src: &'c Currency,
        dst: &'c Currency,
        rate: f64,
        provider: String,
        duration: Option<Duration>,
    ) -> Self {
        let now = Local::now();
        let cache_until = duration.map(|d| now + d);
        Self::new(src, dst, now, rate, provider, cache_until)
    }

    /// A 1:1 rate for a currency and itself
    pub fn parity(c: &'c Currency) -> Self {
        Rate::new(c, c, Local::now(), 1., String::from("PARITY"), None)
    }

    /// Source currency
    pub fn src(&self) -> &Currency {
        &self.src
    }

    /// Destination currency
    pub fn dst(&self) -> &Currency {
        &self.dst
    }

    /// Date of the rate
    pub fn date(&self) -> &DateTime<LocalTime> {
        &self.date
    }

    /// Rate
    pub fn rate(&self) -> f64 {
        self.rate
    }

    /// Provider
    pub fn provider(&self) -> &str {
        &self.provider
    }

    /// Cache until
    pub fn cache_until(&self) -> &Option<DateTime<LocalTime>> {
        &self.cache_until
    }

    pub fn uptodate(&self, now: DateTime<LocalTime>) -> bool {
        match self.cache_until() {
            Some(date) => date > &now,
            None => false,
        }
    }
}
