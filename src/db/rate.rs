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

//! Structs related to rate and rate storage

use std::convert::TryFrom;
use std::fmt;

use serde_derive::{Deserialize, Serialize};

use chrono::offset::Local as LocalTime;
use chrono::prelude::*;

use crate::currency;
use crate::rate::Rate;

/// RateInternal is a trimmed down version of Rate to map to the db schema.
/// Fields have the same meaning as Rate
#[derive(Clone, PartialOrd, PartialEq, Debug, Serialize, Deserialize)]
pub(super) struct RateInternal {
    pub(super) src: String,
    pub(super) dst: String,
    pub(super) date: DateTime<LocalTime>,
    pub(super) rate: f64,
    pub(super) provider: String,
    pub(super) cache_until: DateTime<LocalTime>,
}

impl TryFrom<RateInternal> for Rate<'static> {
    type Error = RateInternalConversionError;

    fn try_from(value: RateInternal) -> Result<Rate<'static>, Self::Error> {
        match value {
            RateInternal {
                src,
                dst,
                date,
                rate,
                provider,
                cache_until,
            } => {
                let src = currency::existing_from_iso(&src)
                    .ok_or(RateInternalConversionError::CurrencyNotFound)?;
                let dst = currency::existing_from_iso(&dst)
                    .ok_or(RateInternalConversionError::CurrencyNotFound)?;
                Ok(Rate::new(src, dst, date, rate, provider, Some(cache_until)))
            }
        }
    }
}

impl<'r> TryFrom<&Rate<'r>> for RateInternal {
    type Error = RateInternalConversionError;

    fn try_from(value: &Rate<'r>) -> Result<Self, Self::Error> {
        Ok(RateInternal {
            src: value.src().get_main_iso().to_string(),
            dst: value.dst().get_main_iso().to_string(),
            date: value.date().clone(),
            rate: value.rate(),
            provider: value.provider().to_string(),
            cache_until: value
                .cache_until()
                .ok_or(RateInternalConversionError::MissingCacheUntil)?,
        })
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum RateInternalConversionError {
    MissingCacheUntil,
    CurrencyNotFound,
}

impl fmt::Display for RateInternalConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RateInternalConversionError::MissingCacheUntil => write!(
                f,
                "Missing cache until field in the rate, can’t be stored in the database"
            ),
            RateInternalConversionError::CurrencyNotFound => write!(
                f,
                "Unknown currency, rate can’t be read from the database"
            ),
        }
    }
}

impl std::error::Error for RateInternalConversionError {}
