#[allow(unused_variables)]
use rusqlite::*;
use zstd::*;
use base64::*;
use serde::{Deserialize, Serialize};
use stark_hash::Felt;

#[derive(Debug,Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct EventData(pub Felt);

#[derive(Debug,Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct EventKey(pub Felt);
fn main() {

    let mut id:i64 = 0;
    let mut block_number:i64 = 0;
    let mut idx:i64 = 0;
    let mut transaction_hash:String = String::from("s");
    let mut from_address:String = String::from("value");
    // let mut keys:String = String::from("s");
    let mut data: String = String::from("test");
    { // open for db work
        let db = Connection::open("mainnet.sqlite").expect("db conn fail");
        let mut receiver = db
            .prepare("SELECT * FROM starknet_events")
            .expect("receiver failed");
        let mut rows = receiver
            .query(named_params!{})
            .expect("rows failed");
            
        while let Some(row) = rows.next().expect("while row failed") {
            id=row.get(0).expect("get row failed");
            block_number=row.get(1).expect("get block number row failed");
            idx=row.get(2).expect("get idx row failed");
            let transaction_hash = row.get_ref_unwrap("transaction_hash").as_blob().unwrap();
            let transaction_hash: Vec<_> = transaction_hash
                    .chunks_exact(32)
                    .map(|transaction_hash| {
                        let transaction_hash = Felt::from_be_slice(transaction_hash).unwrap();
                        EventData(transaction_hash)
                    })
                    .collect();
                    println!("transaction_hash {:?}",transaction_hash);
            // // from_address=row.get(4).expect("get from address row failed");
            // let keys = row.get_ref_unwrap("keys").as_str().unwrap();

            // // no need to allocate a vec for this in loop
            // let mut temp = [0u8; 32];

            // let keys: Vec<_> = keys
            //     .split(' ')
            //     .map(|key| {
            //         let used =
            //             base64::decode_config_slice(key, base64::STANDARD, &mut temp).unwrap();
            //         let key = Felt::from_be_slice(&temp[..used]).unwrap();
            //         EventKey(key)
            //     })
            //     .collect();
            // println!("keys {:?}",keys);
            let data = row.get_ref_unwrap("data").as_blob().unwrap();
                let data: Vec<_> = data
                    .chunks_exact(32)
                    .map(|data| {
                        let data = Felt::from_be_slice(data).unwrap();
                        EventData(data)
                    })
                    .collect();
                    println!("data {:?}",data[0]);
               

        }
    } // close db work
    println!("id : {}", id);
    // println!("Block Number : {}", keys);
    println!("idx : {}", idx);
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}
