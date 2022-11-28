use core::panic;
use std::{net::Ipv4Addr, env, str::FromStr};
use log::{error, warn, info, debug, trace, LevelFilter};

use lazy_static::lazy_static;

fn var_not_defined(var: &str) -> String {
    panic!("[{}] -- {} environment variable not defined", "Main", var)
}

pub fn check_vars() {
    lazy_static::initialize(&DISCORD_TOKEN);
    lazy_static::initialize(&SHARD_NB);
}

lazy_static! {
    // Discord
    pub static ref DISCORD_TOKEN: String = env::var("DISCORD_TOKEN").unwrap_or_else(|_e| {
        var_not_defined("DISCORD_TOKEN")
    });

    pub static ref SHARD_NB: u64 = env::var("SHARD_NB").unwrap_or_else(|_e| {
        var_not_defined("SHARD_NB")
    }).parse().unwrap_or_else(|e| {
        panic!("Can't parse SHARD_NB {}", e);
    });

}