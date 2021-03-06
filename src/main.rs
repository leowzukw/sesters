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

use clap::ArgMatches;
use kv::{Config as KvConfig, Manager};
use log::{error, info};
use std::io::stdout;

mod api;
mod clap_def;
mod config;
mod convert;
mod currency;
mod db;
mod price_format;
mod price_in_text;
mod rate;

use crate::config::Config;
use crate::currency::Currency;
use crate::db::Db;

/// Main context to pass what is initiliazed in this module and what is parsed
/// in global tags
pub(crate) struct MainContext<'mc> {
    db: Db,
    destination_currencies: Vec<&'mc Currency>,
    cfg: Config,
}

fn from_args(matches: ArgMatches) {
    let mut out = stdout();
    let cfg = Config::get().unwrap();

    // Manager for the database
    let mut mgr = Manager::new();
    info!("Initialize database");
    let kcfg = KvConfig::default(&cfg.db_path);
    let db = Db::new(kcfg, &mut mgr);

    // Argument parsing
    let currency_iso_names_cfg: Vec<&str> = cfg.currencies.iter().map(|s| s.as_str()).collect();
    let currency_iso_names: Vec<&str> = matches
        .values_of("TO")
        .map_or(currency_iso_names_cfg, |to| to.collect());
    let destination_currencies = currency_iso_names
        .iter()
        .filter_map(|iso_name| {
            currency::existing_from_iso(&iso_name).or_else(|| {
                error!("Invalid currency iso symbol '{}', ignored", iso_name);
                None
            })
        })
        .collect();

    let ctxt = MainContext {
        db,
        cfg,
        destination_currencies,
    };

    match matches.subcommand() {
        ("convert", Some(m)) => crate::convert::run(ctxt, m),
        (_, _) => crate::clap_def::get_app()
            .write_long_help(&mut out)
            .expect("failed to write to stdout"),
    }

    info!("Exiting");
}

fn main() {
    log::set_max_level(log::LevelFilter::Info);
    env_logger::init();
    info!("Starting up");

    from_args(crate::clap_def::get_app().get_matches());
}
