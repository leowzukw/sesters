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

use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};

use std::fmt;

use crate::rate::Rate;

#[cfg(test)]
mod tests {
    use super::*;

    // Test static currencies
    #[test]
    fn static_currency_check() {
        for c in ALL_CURRENCIES.iter() {
            assert!(c.check());
        }
    }

    #[test]
    fn static_currency_iso() {
        for c in ALL_CURRENCIES.iter() {
            assert_eq!(existing_from_iso(c.get_main_iso()), Some(c));
        }
        assert_eq!(existing_from_iso("___"), None);
    }
}

/// Position of a symbol against an amount
#[derive(Debug, PartialOrd, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum Pos {
    Before,
    After,
}

impl Default for Pos {
    fn default() -> Pos {
        Pos::After
    }
}

/// An association between currency & amount, TODO with a position
#[derive(Debug, Clone, PartialEq)]
pub struct PriceTag<'c> {
    currency: &'c Currency,
    amount: f64,
    // TODO /// Position of the currency indicator against amount
    // position: Pos,
}

impl<'c> PriceTag<'c> {
    /// Create new amount associated to a currency
    pub fn new(currency: &'c Currency, amount: f64) -> Self {
        Self { currency, amount }
    }

    /// Get currency of the amount
    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    // TODO Place this method with Rate structure to avoid having a rate method
    /// Convert the amount (in src currency) to an amount (in a dest currency).
    /// The relation from the currency to the other is given by a rate.
    pub fn convert<'a, 'r>(
        &'a self,
        rate: &'r Rate<'c>,
    ) -> Result<PriceTag<'r>, ConversionError<'a, 'c, 'r>> {
        if self.currency != rate.src() {
            Err(ConversionError::new(rate, &self))
        } else {
            Ok(PriceTag::new(rate.dst(), rate.rate() * self.amount))
        }
    }
}

impl<'c> fmt::Display for PriceTag<'c> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO Use symbol, proper separator (, or .), proper number of cents (usually 2 or 3)
        write!(f, "{} {:.*}", self.currency.get_main_iso(), 2, self.amount)
    }
}

/// Error when converting an amount from a currency to another. Record source currency and Rate
#[derive(Debug, Clone)]
pub struct ConversionError<'a, 'c, 'r> {
    rate: &'r Rate<'c>,
    amount: &'a PriceTag<'c>,
}

impl<'a, 'c, 'r> ConversionError<'a, 'c, 'r> {
    /// New conversion error
    pub fn new(rate: &'r Rate<'c>, amount: &'a PriceTag<'c>) -> Self {
        ConversionError { rate, amount }
    }
}

/// Represent a currency like US Dollar or Euro, with its symbols
// TODO Improve serialization/deserialization
#[derive(Debug, Default, PartialOrd, PartialEq, Clone)]
pub struct Currency {
    /// Symbols, like ₿, ฿ or Ƀ for Bitcoin. Slice must not be empty
    symbols: &'static [&'static str],
    /// ISO4217-ish symbol, like BTC or XBT for Bitcoin. Slice must not be empty
    isos: &'static [&'static str],
    /// Human name(s). Slice must not be empty
    names: &'static [&'static str],
    /// Position to display symbols
    pos: Pos,
}

impl Currency {
    pub fn isos(&self) -> &'static [&'static str] {
        &self.isos
    }

    /// Main iso symbol for a currency, USD for instance
    pub fn get_main_iso(&self) -> &str {
        &self.isos[0]
    }

    pub fn names(&self) -> &'static [&'static str] {
        &self.names
    }

    pub fn symbols(&self) -> &'static [&'static str] {
        &self.symbols
    }

    pub fn pos(&self) -> Pos {
        self.pos
    }

    /// Constructor, copies the &str given. Panics if vectors are empty TODO Use Result type instead
    pub fn new(
        symbols: &'static [&'static str],
        isos: &'static [&'static str],
        names: &'static [&'static str],
        pos: Pos,
    ) -> Currency {
        let c = Currency {
            symbols,
            isos,
            names,
            pos,
        };
        assert!(c.check());
        c
    }

    /// Check if a currency is conform to the constraints listed in the definition of the structure
    pub fn check(&self) -> bool {
        !self.symbols.is_empty() && !self.isos.is_empty() && !self.names.is_empty()
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_main_iso())
    }
}

/// Some common currency
/// Symbols and ISO are from Wikipedia
// TODO Use static instead of const

/// https://en.wikipedia.org/wiki/Bitcoin
// TODO Use Currency::new once const fn is in stable
pub const BTC: Currency = Currency {
    symbols: &["₿", "฿", "Ƀ"],
    isos: &["BTC", "XBT"],
    names: &["Bitcoin"],
    pos: Pos::After,
};

/// https://en.wikipedia.org/wiki/United_States_dollar
pub const USD: Currency = Currency {
    symbols: &["$"],
    isos: &["USD"],
    names: &["United States dollar"],
    pos: Pos::Before,
};

/// https://en.wikipedia.org/wiki/Euro
pub const EUR: Currency = Currency {
    symbols: &["€"],
    isos: &["EUR"],
    names: &["Euro"],
    pos: Pos::After,
};

/// https://en.wikipedia.org/wiki/Pound_sterling
pub const GBP: Currency = Currency {
    symbols: &["£"],
    isos: &["GBP"],
    names: &["Pound sterling"],
    pos: Pos::Before,
};

/// https://en.wikipedia.org/wiki/Swiss_franc
pub const CHF: Currency = Currency {
    symbols: &["CHF", "Fr.", "SFr.", "Fr.sv.", "₣"],
    isos: &["CHF"],
    names: &["Swiss Franc"],
    pos: Pos::Before,
};

/// https://en.wikipedia.org/wiki/Japanese_yen
pub const JPY: Currency = Currency {
    symbols: &["¥", "円", "圓"],
    isos: &["JPY"],
    names: &["Yen"],
    pos: Pos::Before,
};

lazy_static! {
    /// All currencies registered
    pub static ref ALL_CURRENCIES: Vec<Currency> = vec![
        BTC, USD, EUR, GBP, CHF, JPY
    ];
}

/// Get an existing currency from ISO code
pub fn existing_from_iso(code: &str) -> Option<&'static Currency> {
    match code {
        "BTC" => Some(&BTC),
        "USD" => Some(&USD),
        "EUR" => Some(&EUR),
        "GBP" => Some(&GBP),
        "CHF" => Some(&CHF),
        "JPY" => Some(&JPY),
        _ => None,
    }
}
