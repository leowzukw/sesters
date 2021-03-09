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

//! Tests for the database

use chrono::Duration;

use crate::currency::{BTC, CHF, EUR, JPY};

use super::*;

#[test]
fn migrations_test() {
    assert!(MIGRATIONS.validate().is_ok());
}

// TODO Test 3 functions on rate, with rate.cache_until = Some() / None

// Rate with cache_until == None
fn rate_cun() -> Rate<'static> {
    Default::default()
}

// Rate with cache_until == Some(date) where date is in the future
fn rate_cus_future() -> Rate<'static> {
    Rate::now(
        &JPY,
        &BTC,
        2777277.,
        String::from("kraken"),
        Some(Duration::weeks(3)),
    )
}

// Rate with cache_until == Some(date) where date is in the past (expired rate)
fn rate_cus_past() -> Rate<'static> {
    Rate::new(
        &CHF,
        &EUR,
        Local::now() - Duration::weeks(16),
        0.9,
        String::from("xe"),
        Some(Local::now() - Duration::weeks(15)),
    )
}

// Ensure we can convert back and forth
#[test]
fn rate_convert_back_forth_test() {
    for rate in vec![rate_cus_future(), rate_cus_past()] {
        let db = Db::new_in_memory().unwrap();
        assert!(dbg!(db.set_rate(&rate)).is_ok());

        let (mut retrieved_rates_outdated, mut retrieved_rates_uptodate) = db
            .get_rates(rate.src(), rate.dst(), rate.provider())
            .unwrap();
        let mut retrieved_rates = Vec::new();
        retrieved_rates.append(&mut retrieved_rates_outdated);
        retrieved_rates.append(&mut retrieved_rates_uptodate);

        assert_eq!(retrieved_rates.len(), 1);
        assert_eq!(rate, retrieved_rates[0]);
    }
}

// Reject cache_until == None
#[test]
fn cache_until_none_rejected_test() {
    unimplemented!()
}

// Ensure uptodate and outdated rates are returned correctly organized
#[test]
fn outdated_uptodate_test() {
    for rate in vec![rate_cun(), rate_cus_future(), rate_cus_past()] {
        let db = Db::new_in_memory().unwrap();
        assert!(dbg!(db.set_rate(&rate)).is_ok());
        let (rates_outdated, rates_uptodate) = db
            .get_rates(rate.src(), rate.dst(), rate.provider())
            .unwrap();

        for ru in rates_uptodate {
            assert!(ru.uptodate(Local::now()))
        }
        for ro in rates_outdated {
            assert!(!ro.uptodate(Local::now()))
        }
    }
}

#[test]
fn init_do_test() {
    let _ = Db::new(&Config::new().unwrap()).unwrap();
}
