use std::env;

use anyhow::{Context, Result};
use flux::BulkRepository;
use flux_derive::{Entity, MongoEntity};
use flux_mongodb::{MongoObjectId, MongoRepository};
use mongodb::{
    bson::{doc, oid::ObjectId},
    Client,
};

#[derive(Clone, Debug, Entity, MongoEntity)]
#[collection_name = "cookbook_mongo_bulk_products"]
struct Product {
    #[primary_key]
    id: MongoObjectId,
    name: String,
    price_cents: i32,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let database = connect_mongo().await?;
    database
        .collection::<mongodb::bson::Document>("cookbook_mongo_bulk_products")
        .delete_many(doc! {})
        .await
        .context("clear cookbook_mongo_bulk_products")?;

    let repo = MongoRepository::<Product>::new(database);
    let products = (0..25)
        .map(|index| Product {
            id: MongoObjectId(ObjectId::new()),
            name: format!("mongo-bulk-product-{index}"),
            price_cents: 1_000 + index,
        })
        .collect::<Vec<_>>();

    let inserted = repo
        .insert_many(&products)
        .await
        .context("Mongo insert_many")?;
    let updated = inserted
        .into_iter()
        .map(|mut product| {
            product.price_cents += 100;
            product
        })
        .collect::<Vec<_>>();
    let saved = repo.save_many(&updated).await.context("Mongo save_many")?;
    let ids = saved.iter().map(|product| product.id).collect::<Vec<_>>();
    let deleted = repo.delete_many(&ids).await.context("Mongo delete_many")?;

    assert_eq!(deleted, ids.len() as u64);

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
