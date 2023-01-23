#[allow(unused_variables)]
use rusqlite::*;
mod class;
use std::time::{Duration, Instant};
use pathfinder_common::{ EventKey,felt};
mod indexer;
use serde::Deserialize;

#[derive(Clone,serde::Deserialize, Debug, PartialEq, Eq)]
pub struct GetEventsInput {
    filter: EventFilter,
}

/// Contains event filter parameters passed to `starknet_getEvents`.
#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EventFilter {
    #[serde(default)]
    pub keys: Vec<EventKey>,
}

fn main() {
    { // open for db work
        let mut db = Connection::open("mainnet.sqlite").expect("db conn fail");
        let start = Instant::now();
             let filter =&indexer::StarknetEventFilter {
            keys: vec![EventKey(felt!("0x0099CD8BDE557814842A3121E8DDFD433A539B8C9F14BF31EBF108D12E6196E9"))],
        };
        let tx = db.transaction().unwrap();

        let events = indexer::StarknetEventsTable::get_events(&tx, filter);
        let duration = start.elapsed();
        println!("res {:?}", events);
        println!("Time elapsed in getContract is: {:?}", duration);

       
    } // close db work

}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}
