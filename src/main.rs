#[allow(unused_variables)]
use rusqlite::*;
use pathfinder_common::{felt, EventKey};
use std::time::{Duration, Instant};
use moso_events::{StarknetEventFilter, StarknetEmittedEvent, StarknetEventsTable};
use serde::Deserialize;
use pathfinder_database::{MosoDb};
#[derive(Clone, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct GetEventsInput {
    filter: EventFilter,
}
use pathfinder_common::StarknetBlockNumber;
/// Contains event filter parameters passed to `starknet_getEvents`.
#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EventFilter {
    #[serde(default)]
    pub keys: Vec<EventKey>,
}

#[tokio::main]
async fn main() {
    {
        let mut db = Connection::open("mainnet_18617.sqlite").expect("db conn fail");
        let start = Instant::now();
        const BLOCK_NUMBER: usize = 2375;
        const TO_BLOCK_NUMBER: usize = 2375;

        let filter = &StarknetEventFilter {
            from_block: Some(StarknetBlockNumber::new_or_panic(BLOCK_NUMBER as u64)),
            to_block: Some(StarknetBlockNumber::new_or_panic(TO_BLOCK_NUMBER as u64)), 
            keys: vec![EventKey(felt!(
                "0x99cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9" // Transfer event key
            )),
            EventKey(felt!(
                "0x182d859c0807ba9db63baf8b9d9fdbfeb885d820be6e206b9dab626d995c433" // TransferSingle event key
            )),
            EventKey(felt!(
                "0x2563683c757f3abe19c4b7237e2285d8993417ddffe0b54a19eb212ea574b08" // TransferBatch event key
            ))
            ],                                                                    
        };
        let tx = db.transaction().unwrap();

        let events = StarknetEventsTable::get_events(&tx, filter).unwrap();
        let events = events.events;
        let db = MosoDb::init().await;
        let events = MosoDb::insert_events(&db, events).await;
        let duration = start.elapsed();
        println!("res {:?}", events);
        println!("Time elapsed in getContract is: {:?}", duration);
    } 
}
