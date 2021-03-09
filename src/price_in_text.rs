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

//! A module to find currency unit with amount (a **price tag**) in raw text

use itertools::Itertools;
use log::{debug, trace};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::convert::TryInto;
use std::ops::Bound::Included;

use crate::currency;
use crate::currency::{Currency, PriceTag};
use crate::price_format::PriceFormat;
use regex::{Regex, RegexBuilder};

#[cfg(test)]
mod tests;

// Information about a price tag match, will be used to compute the probability
// of assocation between an amount and a currency
#[derive(Debug, Clone)]
pub struct PriceTagMatch<'c> {
    // Amount of the currency
    amount: f64,
    // Currency matching
    currency: &'c Currency,
    // Absolute distance between symbol and amount
    distance: i32,
    // Whether the order between amount and symbol is conform to currency property
    correct_symbol_order: bool,
}

/// A PriceTagMatch is better than another if the distance between amount and
/// symbol is shorter. In case where the distances are the same, the
/// PriceTagMatch which has the correct_symbol_order is better.
///
/// With this ordering, we try to make the best match the smallest: as the best
/// match would have a distance of 0 with the right order, it is the minimum of
/// the set of all PriceTagMatch.
///
/// It should be noted that we can’t say that two PriceTagMatch are equal if
/// they have different PriceTag. Thus, we only define partial ordering.
impl<'c> PartialOrd for PriceTagMatch<'c> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let o = self.distance.cmp(&other.distance);
        match o {
            Ordering::Less | Ordering::Greater => Some(o),
            Ordering::Equal => match (self.correct_symbol_order, other.correct_symbol_order) {
                (true, true) | (false, false) => {
                    if self == other {
                        Some(Ordering::Equal)
                    } else {
                        None
                    }
                }
                (true, false) => Some(Ordering::Less),
                (false, true) => Some(Ordering::Greater),
            },
        }
    }
}

/// PartialEq consistent with the PartialOrd we defined
impl<'c> PartialEq for PriceTagMatch<'c> {
    fn eq(&self, other: &Self) -> bool {
        self.amount == other.amount
            && *self.currency == *other.currency
            && self.distance == other.distance
            && self.correct_symbol_order == other.correct_symbol_order
    }
}

impl<'c> PriceTagMatch<'c> {
    fn new(
        amount: f64,
        currency: &'c Currency,
        distance: i32,
        correct_symbol_order: bool,
    ) -> PriceTagMatch {
        PriceTagMatch {
            amount,
            correct_symbol_order,
            currency,
            distance,
        }
    }
}

impl<'c> From<PriceTagMatch<'c>> for PriceTag<'c> {
    fn from(cm: PriceTagMatch<'c>) -> Self {
        Self::new(cm.currency, cm.amount)
    }
}

/// Price tag engine, used to extract price tags in plain text
/// It proceeds in 3 steps:
/// 1. Find positions of all number (possibly with various separator)
/// 2. Find positions of currencies looked for, and for each, look for number,
///    forward and backward in a certain distance (name *window*). A
///    probability of “matching” is computed for each.
/// 3. Return N topmost matches or all of them
#[derive(Debug)]
pub struct Engine<'c> {
    options: EngineOptions<'c>,
    /// To match and extract prices in plain text format
    price_match: PriceFormat,
    /// Regular expression to match currency symbol or iso in plain text format, by currency main iso
    currency_matches: HashMap<&'c str, Regex>,
}

impl<'c> Engine<'c> {
    pub fn new() -> Result<Engine<'c>, EngineError> {
        let e = EngineBuilder::new().fire();
        trace!("Engine::new {:?}", e);
        e
    }

    // TODO Return an iterator to lazily cut evaluation
    /// Return all price tag matches found in plain_text
    fn find<'txt>(&self, plain_text: &'txt str) -> Vec<PriceTagMatch> {
        // Record locations of price ends in price tags
        let price_locations = || {
            debug!("computing price_locations…");
            let mut price_loc_start = BTreeMap::new();
            let mut price_loc_end = BTreeMap::new();

            for price_match in &self.price_match.captures_iter(plain_text) {
                price_loc_start.insert(price_match.start(), price_match.price());
                price_loc_end.insert(price_match.end(), price_match.price());
            }
            debug!("price_loc_start: {:?}", price_loc_start);
            trace!("price_loc_end: {:?}", price_loc_end);
            (price_loc_start, price_loc_end)
        };

        let (price_loc_start, price_loc_end) = price_locations();

        let mut pricetag_matches = Vec::new();
        for (currency_main_iso, currency_match) in &self.currency_matches {
            debug!("Matches for {}", currency_main_iso);
            let currency = currency::existing_from_iso(currency_main_iso).unwrap();
            for cap in currency_match.captures_iter(plain_text) {
                let m = cap.get(0);
                trace!("m, before unwrapping: {:?}", m);
                let m = cap.get(0).unwrap(); // 0 is the whole pattern, always present
                let (start, end, win) = (m.start(), m.end(), self.options.window_size);
                trace!("start, end, win: {}, {}, {}", start, end, win);
                let win_before_start = if win > start { 0 } else { start - win };
                // Look backward, for the end of the price. If we were looking
                // from the start of the price, we would miss some corner
                // cases, like this one:
                //     window_size
                //   /-------------\
                //133  Lorem ipsumm USD
                // Perform backward or forward look, depending of the parameters
                use currency::Pos;
                trace!(
                    "before forward look, pricetag_matches: {:?}",
                    pricetag_matches
                );
                let mut look = |location: usize, price: f64, expected_position: Pos| {
                    trace!("&location, &price: {:?}, {:?}", &location, &price);
                    let distance = if expected_position == Pos::Before {
                        (start - location) as i32
                    } else {
                        (location - end) as i32
                    };
                    let ptm = PriceTagMatch::new(
                        price,
                        currency,
                        distance.try_into().unwrap(),
                        currency.pos() == expected_position,
                    );
                    pricetag_matches.push(ptm);
                };
                for (location, price) in
                    price_loc_end.range((Included(&win_before_start), Included(&start)))
                {
                    look(*location, *price, Pos::Before);
                }
                trace!("Looking backward now…");
                // Idem, but with the start of the number when looking forward
                for (location, price) in
                    price_loc_start.range((Included(&end), Included(&(end + win))))
                {
                    look(*location, *price, Pos::After);
                }
                debug!(
                    "after forward and backward look, pricetag_matches: {:?}",
                    pricetag_matches
                );
            }
        }

        pricetag_matches.sort_by_key(|ptm| (ptm.distance, ptm.correct_symbol_order));
        pricetag_matches
    }

    /// Return all price tag found in plain_text
    pub fn all_price_tags<'txt>(&self, plain_text: &'txt str) -> Vec<PriceTag> {
        self.find(plain_text)
            .into_iter()
            .map(|ptm| ptm.into())
            .collect()
    }

    /// Return the top `n` price tags
    pub fn top_price_tags(&self, n: usize, plain_text: &str) -> Vec<PriceTag> {
        self.find(plain_text)
            .into_iter()
            .take(n)
            .map(|ptm| ptm.into())
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct EngineOptions<'c> {
    window_size: usize,
    currencies: &'c [Currency],
    by_symbol: bool,
    by_iso: bool,
    case_insensitive: bool,
    price_format: PriceFormat,
}

impl<'c> Default for EngineOptions<'c> {
    fn default() -> EngineOptions<'c> {
        let currencies: &'c [Currency] = &(*currency::ALL_CURRENCIES);
        EngineOptions {
            window_size: 10,
            currencies,
            by_symbol: true,
            by_iso: true,
            case_insensitive: true,
            // TODO Try to avoid clone call here
            price_format: crate::price_format::COMMON.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EngineBuilder<'c>(EngineOptions<'c>);

impl<'c> EngineBuilder<'c> {
    /// Create a builder with default option, to be custumized
    pub fn new() -> EngineBuilder<'c> {
        EngineBuilder(Default::default())
    }

    /// Consume Builder and fire the Engine, so that it be used to match text
    pub fn fire(self) -> Result<Engine<'c>, EngineError> {
        let price_match = self.0.price_format.clone();

        let mut currency_matches = HashMap::new();
        for currency in self.0.currencies {
            let mut alternatives_slices: Vec<&[&str]> = Vec::new();
            let mut currency_match_string = String::new();
            let mut add_regex_altenatives = |new_alternatives: &'static [&'static str]| {
                trace!(
                    "add_regex_altenatives: alternatives (before): {:?}",
                    &alternatives_slices
                );
                trace!(
                    "add_regex_altenatives: new_alternatives (after): {:?}",
                    &new_alternatives
                );
                alternatives_slices.push(new_alternatives);
                trace!(
                    "add_regex_altenatives: alternatives (after): {:?}",
                    &alternatives_slices
                );
            };
            if self.0.by_iso {
                add_regex_altenatives(currency.isos());
            }
            if self.0.by_symbol {
                add_regex_altenatives(currency.symbols());
            }
            // Escape currency symbols and iso by inserting litteral unicode in the regex
            let alternatives_unicode_escaped = alternatives_slices
                .into_iter()
                .map(|a| a.iter().map(|s| format!("{}", s.escape_unicode())));
            for s in Itertools::intersperse(alternatives_unicode_escaped
                .flatten()
                , "|".to_owned())
            {
                currency_match_string.push_str(&s);
            }
            let currency_match_err = RegexBuilder::new(currency_match_string.as_str())
                .case_insensitive(self.0.case_insensitive)
                .build();
            match currency_match_err {
                Ok(currency_match) => {
                    currency_matches.insert(currency.get_main_iso(), currency_match)
                }
                Err(err) => return Err(EngineError::CurrencyMatchRegex(err)),
            };
            debug!("currency_matches: {:?}", currency_matches)
        }

        Ok(Engine {
            options: self.0,
            price_match,
            currency_matches,
        })
    }

    /// Set the size of the window used as the distance between a price and a
    /// currency unit when looking for price tag
    pub fn window(&mut self, size: usize) -> &mut EngineBuilder<'c> {
        self.0.window_size = size;
        self
    }

    /// Set the currency set to look for
    pub fn currencies(&mut self, currencies: &'c [Currency]) -> &mut EngineBuilder<'c> {
        self.0.currencies = currencies;
        self
    }

    /// Find price tag using the symbol of the currency, like “€” or “$”
    pub fn by_symbol(&mut self, yes: bool) -> &mut EngineBuilder<'c> {
        self.0.by_symbol = yes;
        self
    }

    /// Find symbol case insensitively
    pub fn case_insensitive(&mut self, yes: bool) -> &mut EngineBuilder<'c> {
        self.0.case_insensitive = yes;
        self
    }

    /// Find price tag using the iso of the currency, like “EUR” or “USD”
    pub fn by_iso(&mut self, yes: bool) -> &mut EngineBuilder<'c> {
        self.0.by_iso = yes;
        self
    }

    /// Set the PriceFormat used to match and extract prices in plain text
    pub fn price(&mut self, format: PriceFormat) -> &mut EngineBuilder<'c> {
        self.0.price_format = format;
        self
    }
}

/// Error that occured while building and firing the engine
#[derive(Clone, PartialEq, Debug)]
#[non_exhaustive]
pub enum EngineError {
    /// Invalid regex for currency_match
    CurrencyMatchRegex(regex::Error),
}
