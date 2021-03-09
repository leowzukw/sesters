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

//! Access several API used by Sesters

use chrono::Duration;
use log::{debug, error, trace};
use std::error::Error;

use crate::config::Config;
use crate::currency::Currency;
use crate::rate::Rate;

use reqwest::{Client, RequestBuilder, Response};
use std::collections::HashMap;

/// Trait common to all supported API endpoints
pub trait RateApi {
    /// Initialise the rate API struct with config, as it may contain API key
    fn new(config: &Config) -> Self;

    // TODO Add method to get possible conversion and store it in initial
    // struct. This requires passing client to new

    /// Provider identifier, should be based on provider url
    fn provider_id(&self) -> String;

    /// Build the query to get rate from currency src to currency dst
    fn rate_query<'c>(
        &self,
        client: &Client,
        src: &'c Currency,
        dst: &'c Currency,
    ) -> RequestBuilder;

    /// Treat result of the query to get a rate
    fn treat_result<'c>(
        &self,
        res: Response,
        src: &'c Currency,
        dst: &'c Currency,
    ) -> Result<Rate<'c>, Box<dyn Error>>;

    /// Perform request to get rate, if it exists
    fn rate<'c>(&self, client: &Client, src: &'c Currency, dst: &'c Currency) -> Option<Rate<'c>> {
        let rate_err = || -> Result<Rate, Box<dyn Error>> {
            debug!("Performing conversion request for {} -> {}", src, dst);
            let res = self.rate_query(client, src, dst).send()?;
            debug!("Conversion request for {} -> {} done", src, dst);
            trace!("Conversion request result: {:?}", &res);
            self.treat_result(res, src, dst)
        };
        match rate_err() {
            Err(e) => {
                error!(
                    "Error while performing request for {} -> {}: {}",
                    src, dst, e
                );
                None
            }
            Ok(rate) => Some(rate),
        }
    }
}

/// For https://currencyconverterapi.com
pub struct CurrencyConverterApiCom {
    /// API key, if any
    #[allow(dead_code)] // TODO Use the key field
    key: String,
}

impl RateApi for CurrencyConverterApiCom {
    // TODO Use config to populate key field
    fn new(_: &Config) -> Self {
        CurrencyConverterApiCom {
            key: "".to_string(),
        }
    }

    fn provider_id(&self) -> String {
        String::from("currencyconverterapi.com")
    }

    fn rate_query<'c>(
        &self,
        client: &Client,
        src: &'c Currency,
        dst: &'c Currency,
    ) -> RequestBuilder {
        let pair = format!("{0}_{1}", src.get_main_iso(), dst.get_main_iso());
        client
            .get("https://free.currencyconverterapi.com/api/v6/convert")
            .query(&[("q", pair.as_str()), ("compact", "ultra")])
    }

    fn treat_result<'c>(
        &self,
        mut res: Response,
        src: &'c Currency,
        dst: &'c Currency,
    ) -> Result<Rate<'c>, Box<dyn Error>> {
        let pair = format!("{0}_{1}", src.get_main_iso(), dst.get_main_iso());
        // XXX Maybe HashMap is too long to build, Vec would be better
        let rates: HashMap<String, f64> = res.json()?;
        Ok(Rate::now(
            src,
            dst,
            rates[&pair],
            self.provider_id(),
            Some(Duration::hours(1)),
        ))
    }
}

/// For https://exchangeratesapi.io/
pub struct ExchangeRatesApiIo {
    /// API key, if any
    #[allow(dead_code)] // TODO Use the key field
    key: String,
}

impl RateApi for ExchangeRatesApiIo {
    fn new(_: &Config) -> Self {
        ExchangeRatesApiIo {
            key: "".to_string(),
        }
    }

    fn provider_id(&self) -> String {
        String::from("exchangeratesapi.io")
    }

    fn rate_query<'c>(
        &self,
        client: &Client,
        src: &'c Currency,
        _dst: &'c Currency,
    ) -> RequestBuilder {
        client
            .get("https://api.exchangeratesapi.io/latest")
            .query(&[("base", src.get_main_iso())])
    }

    // TODO Use other rates given
    fn treat_result<'c>(
        &self,
        mut res: Response,
        src: &'c Currency,
        dst: &'c Currency,
    ) -> Result<Rate<'c>, Box<dyn Error>> {
        // XXX Maybe HashMap is too long to build, Vec would be better
        let rates: serde_json::Value = res.json()?;
        // TODO Use date provided and remove unwrap, to properly deal with errors
        Ok(Rate::now(
            src,
            dst,
            rates
                .get("rates")
                .unwrap()
                .get(&dst.get_main_iso())
                .unwrap()
                .as_f64()
                .unwrap(),
            self.provider_id(),
            // Updated once a day, let’s bet for a refresh every few hours
            Some(Duration::hours(6)),
        ))
    }
}
