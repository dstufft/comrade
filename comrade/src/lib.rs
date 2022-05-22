pub mod config;
pub mod errors;
pub mod parser;
pub mod watcher;

pub mod meta {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
