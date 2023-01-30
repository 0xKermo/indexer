mod class;
use anyhow::Context;
use class::{ContractAbiEntry, ContractClass};
use pathfinder_common::{
    ContractAddress, EventData, EventKey, StarknetBlockNumber, StarknetTransactionHash,
};
use pathfinder_serde::starkhash_to_dec_str;
use rusqlite::Transaction;
use serde::Serialize;
use stark_hash::Felt;
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum EventType {
    None,
    Mint,
    Burn,
    Transfer,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]

pub enum ContractType {
    ERC721,
    ERC1155,
}

pub struct StarknetEventFilter {
    pub from_block: Option<StarknetBlockNumber>,
    pub to_block: Option<StarknetBlockNumber>,
    pub keys: Vec<EventKey>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StarknetEmittedEvent {
    pub contract_address: ContractAddress,
    from: String,
    to: String,
    token_id: String,
    pub block_number: u64,
    pub transaction_hash: StarknetTransactionHash,
    event_type: EventType,
    contrat_type: ContractType,
}

#[derive(Copy, Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum EventFilterError {
    #[error("requested page size is too big, supported maximum is {0}")]
    PageSizeTooBig(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Events {
    pub events: Vec<StarknetEmittedEvent>,
}

pub struct StarknetEventsTable {}

impl StarknetEventsTable {
    fn encode_event_key_to_base64(key: &EventKey, buf: &mut String) {
        base64::encode_config_buf(key.0.as_be_bytes(), base64::STANDARD, buf);
    }

    fn event_query<'query, 'arg>(
        base: &'query str,
        from_block: Option<&'arg StarknetBlockNumber>,
        to_block: Option<&'arg StarknetBlockNumber>,
        keys: &'arg [EventKey],
        key_fts_expression: &'arg mut String,
    ) -> (
        std::borrow::Cow<'query, str>,
        Vec<(&'static str, &'arg dyn rusqlite::ToSql)>,
    ) {
        let mut base_query = std::borrow::Cow::Borrowed(base);

        let mut where_statement_parts: Vec<&'static str> = Vec::new();
        let mut params: Vec<(&str, &dyn rusqlite::ToSql)> = Vec::new();

        // filter on block range
        match (from_block, to_block) {
            (Some(from_block), Some(to_block)) => {
                where_statement_parts.push("block_number BETWEEN :from_block AND :to_block");
                params.push((":from_block", from_block));
                params.push((":to_block", to_block));
            }
            (Some(from_block), None) => {
                where_statement_parts.push("block_number >= :from_block");
                params.push((":from_block", from_block));
            }
            (None, Some(to_block)) => {
                where_statement_parts.push("block_number <= :to_block");
                params.push((":to_block", to_block));
            }
            (None, None) => {}
        }

        if !keys.is_empty() {
            let needed =
                (keys.len() * (" OR ".len() + "\"\"".len() + 44)).saturating_sub(" OR ".len());
            if let Some(more) = needed.checked_sub(key_fts_expression.capacity()) {
                key_fts_expression.reserve(more);
            }

            let _capacity = key_fts_expression.capacity();

            keys.iter().enumerate().for_each(|(i, key)| {
                key_fts_expression.push('"');
                Self::encode_event_key_to_base64(key, key_fts_expression);
                key_fts_expression.push('"');

                if i != keys.len() - 1 {
                    key_fts_expression.push_str(" OR ");
                }
            });

            debug_assert_eq!(
                _capacity,
                key_fts_expression.capacity(),
                "pre-reservation was not enough"
            );

            base_query.to_mut().push_str(" INNER JOIN starknet_events_keys ON starknet_events.rowid = starknet_events_keys.rowid");
            where_statement_parts.push("starknet_events_keys.keys MATCH :events_match");
            params.push((":events_match", &*key_fts_expression));
        }

        if !where_statement_parts.is_empty() {
            let needed = " WHERE ".len()
                + where_statement_parts.len() * " AND ".len()
                + where_statement_parts.iter().map(|x| x.len()).sum::<usize>();

            let q = base_query.to_mut();
            if let Some(more) = needed.checked_sub(q.capacity() - q.len()) {
                q.reserve(more);
            }

            let _capacity = q.capacity();

            q.push_str(" WHERE ");

            let total = where_statement_parts.len();
            where_statement_parts
                .into_iter()
                .enumerate()
                .for_each(|(i, part)| {
                    q.push_str(part);

                    if i != total - 1 {
                        q.push_str(" AND ");
                    }
                });

            debug_assert_eq!(_capacity, q.capacity(), "pre-reservation was not enough");
        }

        (base_query, params)
    }

    pub fn get_events(
        tx: &Transaction<'_>,
        filter: &StarknetEventFilter,
    ) -> anyhow::Result<Events> {
        let base_query = r#"SELECT
                  block_number,
                  transaction_hash,
                  from_address,
                  cc.definition,
                  data,
                  starknet_events.keys as keys
               FROM starknet_events
                INNER JOIN contracts as c ON (starknet_events.from_address = c.address) 
                INNER JOIN contract_code as cc ON (c.hash = cc.hash) "#;

        let mut key_fts_expression = String::new();

        let (mut base_query, mut params) = Self::event_query(
            base_query,
            filter.from_block.as_ref(),
            filter.to_block.as_ref(),
            &filter.keys,
            &mut key_fts_expression,
        );

        let mut statement = tx.prepare(&base_query).context("Preparing SQL query")?;
        let mut rows = statement
            .query(params.as_slice())
            .context("Executing SQL query")?;

        let mut emitted_events = Vec::new();
        while let Some(row) = rows.next().context("Fetching next event")? {
            let abi = row.get_ref_unwrap("definition").as_blob().unwrap();
            let abi = zstd::decode_all(&*abi).unwrap();
            let class = ContractClass::from_definition_bytes(&abi);
            let abi = class.ok().unwrap().abi.unwrap();
            let block_number = row.get_ref_unwrap("block_number").as_i64().unwrap() as u64;
            let transaction_hash: StarknetTransactionHash = row.get_unwrap("transaction_hash");

            let contract_address: ContractAddress = row.get_unwrap("from_address");
            let data = row.get_ref_unwrap("data").as_blob().unwrap();
            let data: Vec<_> = data
                .chunks_exact(32)
                .map(|data| {
                    let data = Felt::from_be_slice(data).unwrap();
                    EventData(data)
                })
                .collect();
            for x in abi.iter() {
                match x {
                    ContractAbiEntry::Event(value) => match value.name.as_str() {
                        "Transfer" => {
                            let parameter_name = value.data.clone().unwrap();
                            let _name = parameter_name[2].name.as_str();
                            match _name {
                                "_tokenId" => {
                                    let from = &data[0].0;
                                    let to = &data[1].0;
                                    let token_id = starkhash_to_dec_str(&data[2].0);
                                    let mut event = StarknetEmittedEvent {
                                        contract_address,
                                        from: from.to_string(),
                                        to: to.to_string(),
                                        token_id,
                                        block_number,
                                        transaction_hash,
                                        event_type: EventType::None,
                                        contrat_type: ContractType::ERC721,
                                    };
                                    match starkhash_to_dec_str(from).as_str() {
                                        "0" => {
                                            event.event_type = EventType::Mint;
                                        }
                                        _ => {
                                            match starkhash_to_dec_str(to).as_str() {
                                                "0" => {
                                                    event.event_type = EventType::Burn;
                                                }
                                                _ => {
                                                    event.event_type = EventType::Transfer;
                                                }
                                            }
                                        }
                                    }
                                    emitted_events.push(event);
                                }
                                _ => {}
                            }
                        }
                        "TransferSingle" => (),
                        "TransferBatch" => (),
                        _ => (),
                    },
                    _ => (),
                }
            }
        }

        Ok(Events {
            events: emitted_events,
        })
    }
}
