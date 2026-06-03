use std::env;

use anyhow::{Context, Result};
use flux::{BulkRepository, GenericFilter, PageRequest, ReadRepository, WriteRepository};
use flux_derive::{Entity, MongoEntity};
use flux_mongodb::{MongoObjectId, MongoRepository};
use mongodb::{
    bson::{doc, oid::ObjectId},
    Client,
};

#[derive(Clone, Debug, Entity, MongoEntity)]
#[collection_name = "cookbook_customers"]
struct Customer {
    #[primary_key]
    id: MongoObjectId,
    name: String,
    status: String,
    age: i32,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let database = connect_mongo().await?;
    database
        .collection::<mongodb::bson::Document>("cookbook_customers")
        .delete_many(doc! {})
        .await
        .context("clear cookbook_customers")?;

    let repo = MongoRepository::<Customer>::new(database);
    let alice = Customer {
        id: MongoObjectId(ObjectId::new()),
        name: "Alice".to_string(),
        status: "active".to_string(),
        age: 36,
    };
    let bob = Customer {
        id: MongoObjectId(ObjectId::new()),
        name: "Bob".to_string(),
        status: "inactive".to_string(),
        age: 41,
    };

    repo.insert(&alice).await.context("insert Mongo customer")?;
    repo.save(&bob).await.context("upsert Mongo customer")?;

    let filter = GenericFilter::<Customer>::new()
        .eq("status", "active")
        .gte("age", 18);
    let page = repo
        .find_page_with_filter(filter, PageRequest::cursor(10, None))
        .await
        .context("find active Mongo customers")?;

    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].name, "Alice");

    let batch = vec![
        Customer {
            id: MongoObjectId(ObjectId::new()),
            name: "Carol".to_string(),
            status: "active".to_string(),
            age: 29,
        },
        Customer {
            id: MongoObjectId(ObjectId::new()),
            name: "Dave".to_string(),
            status: "active".to_string(),
            age: 33,
        },
    ];
    let saved = repo
        .save_many(&batch)
        .await
        .context("bulk upsert Mongo customers")?;
    let ids = saved.iter().map(|customer| customer.id).collect::<Vec<_>>();
    let deleted = repo
        .delete_many(&ids)
        .await
        .context("bulk delete Mongo customers")?;

    assert_eq!(deleted, 2);

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
