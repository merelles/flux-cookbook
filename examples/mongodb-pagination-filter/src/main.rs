use std::env;

use anyhow::{Context, Result};
use flux::{GenericFilter, OrderDirection, PageRequest, ReadRepository, WriteRepository};
use flux_derive::{Entity, MongoEntity};
use flux_mongodb::{MongoObjectId, MongoRepository};
use mongodb::{
    bson::{doc, oid::ObjectId},
    Client,
};

#[derive(Clone, Debug, Entity, MongoEntity)]
#[collection_name = "cookbook_mongo_events"]
struct Event {
    #[primary_key]
    id: MongoObjectId,
    name: String,
    status: String,
    score: i32,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let database = connect_mongo().await?;
    database
        .collection::<mongodb::bson::Document>("cookbook_mongo_events")
        .delete_many(doc! {})
        .await
        .context("clear cookbook_mongo_events")?;

    let repo = MongoRepository::<Event>::new(database);
    for (index, status) in ["open", "paid", "cancelled"].into_iter().enumerate() {
        let event = Event {
            id: MongoObjectId(ObjectId::new()),
            name: format!("mongo-event-{index}"),
            status: status.to_string(),
            score: 10 + index as i32,
        };
        repo.insert(&event).await.context("seed Mongo event")?;
    }

    let filter = GenericFilter::<Event>::new()
        .gte("score", 10)
        .and_group(|query| {
            query
                .or(|query| query.eq("status", "open"))
                .or(|query| query.eq("status", "paid"))
        })
        .order_by("score", OrderDirection::Desc);
    let page = repo
        .find_page_with_filter(filter, PageRequest::cursor(10, None))
        .await
        .context("find filtered Mongo event page")?;

    assert_eq!(page.items.len(), 2);
    assert!(page.items.iter().all(|event| event.status != "cancelled"));

    Ok(())
}

async fn connect_mongo() -> Result<mongodb::Database> {
    let uri = env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let database_name = env::var("MONGO_DATABASE").unwrap_or_else(|_| "flux_cookbook".to_string());
    let client = Client::with_uri_str(&uri)
        .await
        .context("failed to connect to MongoDB")?;
    client
        .database("admin")
        .run_command(doc! { "ping": 1 })
        .await
        .context("failed to ping MongoDB")?;
    Ok(client.database(&database_name))
}
