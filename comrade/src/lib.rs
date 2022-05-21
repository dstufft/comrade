pub mod errors;
pub mod watcher;

pub mod meta {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
