use mongodb::{bson::doc, options::ClientOptions, Client};
use dotenv::dotenv;
use std::env;
use moso_events::StarknetEmittedEvent;
pub struct MosoDb {
    client: Client,
}
impl MosoDb{
    pub async fn init() -> Self {
        dotenv().ok();
        let uri = match env::var("MONGODBURI") {
            Ok(v) => v.to_string(),
            Err(_) => format!("Error loading env variable"),
        };
        let client = Client::with_uri_str(uri).await.unwrap();
        MosoDb { client }
    }
    pub async fn insert_events(&self, events: Vec<StarknetEmittedEvent>) -> () {
        let db = self.client.database("moso");
        let collection = db.collection("events");
        collection.insert_many(events, None).await.unwrap();
    }

}