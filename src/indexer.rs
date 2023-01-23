use anyhow::Ok;
use pathfinder_common::{
    ContractAddress, EventData, EventKey, StarknetBlockNumber, StarknetTransactionHash,felt
};
use rusqlite::*;
use stark_hash::Felt;
// use starknet_gateway_types::reply::transaction::Event;
use crate::class::{ContractAbiEntry, ContractClass};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StarknetEmittedEvent {
    pub from_address: ContractAddress,
    pub data: Vec<EventData>,
    pub keys: Vec<EventKey>,
    pub block_number: StarknetBlockNumber,
    pub transaction_hash: StarknetTransactionHash,
}
pub struct Indexer {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageOfEvents {
    pub events: Vec<StarknetEmittedEvent>,
}

impl Indexer {
    pub fn get_events(db: &Connection) -> anyhow::Result<PageOfEvents> {
        let mut receiver = db
            .prepare("SELECT * FROM starknet_events")
            .expect("receiver failed");
        let mut rows = receiver.query(named_params! {}).expect("rows failed");

        let mut emitted_events = Vec::new();

        while let Some(row) = rows.next().expect("while row failed") {
            let block_number = row.get_unwrap("block_number");
            let transaction_hash = row.get_unwrap("transaction_hash");
            let from_address = row.get_unwrap("from_address");

            let data = row.get_ref_unwrap("data").as_blob().unwrap();
            let data: Vec<_> = data
                .chunks_exact(32)
                .map(|data| {
                    let data = Felt::from_be_slice(data).unwrap();
                    EventData(data)
                })
                .collect();

            let keys = row.get_ref_unwrap("keys").as_str().unwrap();

            // no need to allocate a vec for this in loop
            let mut temp = [0u8; 32];

            let keys: Vec<_> = keys
                .split(' ')
                .map(|key| {
                    let used =
                        base64::decode_config_slice(key, base64::STANDARD, &mut temp).unwrap();
                    let key = Felt::from_be_slice(&temp[..used]).unwrap();
                    EventKey(key)
                })
                .collect();

            let event = StarknetEmittedEvent {
                data,
                from_address,
                keys,
                block_number,
                transaction_hash,
            };
            println!("event {:?}", event);
        }
        Ok(PageOfEvents {
            events: emitted_events,
        })
    }

    pub fn getContract(db: &Connection) {
        let eventFilter = vec![EventKey(felt!("0x0099CD8BDE557814842A3121E8DDFD433A539B8C9F14BF31EBF108D12E6196E9"))];
        println!("{:?}",eventFilter);
        let mut receiver = db
            .prepare(
                "SELECT se.id,
            se.keys,
            se.from_address,
            c.address,
            cc.abi,
            cc.definition,
            se.data
            from starknet_events as se
            INNER JOIN contracts as c ON (se.from_address = c.address) 
            INNER JOIN contract_code as cc ON (c.hash = cc.hash) 
            WHERE se.keys = :keys",
            )
            .expect("receiver failed");
            let query_values: Vec<_> = eventFilter.iter().map(|x| x as &dyn ToSql).collect();

        let mut rows = receiver.query(&*query_values).expect("rows failed");
        let mut i: u128 = 0;
        
        while let Some(row) = rows.next().expect("while row failed") {
            let abi = row.get_ref_unwrap("definition").as_blob().unwrap();
            println!("{:?}",abi);
            let abi = zstd::decode_all(&*abi).unwrap();
            let class = ContractClass::from_definition_bytes(&abi);
            let abi = class.ok().unwrap().abi.unwrap();
            for x in abi.iter() {
                match x {
                    ContractAbiEntry::Event(value) => match value.name.as_str() {
                        "Transfer" => {
                            println!("{}. Data", i);
                            let keys = row.get_ref_unwrap("keys").as_str().unwrap();

                            // no need to allocate a vec for this in loop
                            let mut temp = [0u8; 32];
                
                            let keys: Vec<_> = keys
                                .split(' ')
                                .map(|key| {
                                    let used =
                                        base64::decode_config_slice(key, base64::STANDARD, &mut temp).unwrap();
                                    let key = Felt::from_be_slice(&temp[..used]).unwrap();
                                    EventKey(key)
                                })
                                .collect();
                                let from_address:ContractAddress = row.get_unwrap("from_address");


                            println!("address {:?}", from_address);
                            println!("keys {:?}", keys);
                            println!("data {:?}", value.data);
                            println!("inputs {:?}", value.inputs);
                            println!("outputs {:?}", value.outputs);
                            println!("----------");
                            let data = row.get_ref_unwrap("data").as_blob().unwrap();
                            let data: Vec<_> = data
                                .chunks_exact(32)
                                .map(|data| {
                                    let data = Felt::from_be_slice(data).unwrap();
                                    EventData(data)
                                })
                                .collect();
                        }
                        _ => (),
                    },
                    _ => (),
                }
            }

            i += 1;
        }
    }

    fn print_type_of<T>(_: &T) {
        println!("{}", std::any::type_name::<T>())
    }
}
