use std::{env, sync::Arc};

use anyhow::{Context, Result};
use flux::{BulkRepository, ReadRepository};
use flux_derive::{Entity, SqlEntity};
use flux_postgres::PostgresRepository;
use tokio_postgres::NoTls;
use uuid::Uuid;

#[derive(Clone, Debug, Entity, SqlEntity)]
#[table_name = "cookbook_bulk_products"]
struct BulkProduct {
    #[primary_key]
    product_id: Uuid,
    name: String,
    price_cents: i32,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let (client, connection) = connect_postgres().await?;
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            eprintln!("postgres connection error: {error}");
        }
    });

    client
        .batch_execute(
            r#"
            CREATE TABLE IF NOT EXISTS cookbook_bulk_products (
                product_id UUID PRIMARY KEY,
                name TEXT NOT NULL,
                price_cents INTEGER NOT NULL
            );
            "#,
        )
        .await
        .context("failed to create cookbook_bulk_products")?;

    let repo = PostgresRepository::<BulkProduct>::new(Arc::new(client));
    let products = (0..25)
        .map(|index| BulkProduct {
            product_id: Uuid::new_v4(),
            name: format!("bulk-product-{index}"),
            price_cents: 1_000 + index,
        })
        .collect::<Vec<_>>();

    let inserted = repo.insert_many(&products).await.context("bulk insert")?;
    let updated = inserted
        .into_iter()
        .map(|mut product| {
            product.price_cents += 100;
            product
        })
        .collect::<Vec<_>>();
    let saved = repo.save_many(&updated).await.context("bulk upsert")?;
    let ids = saved
        .iter()
        .map(|product| product.product_id)
        .collect::<Vec<_>>();
    let deleted = repo.delete_many(&ids).await.context("bulk delete")?;

    assert_eq!(deleted, ids.len() as u64);
    let _remaining = repo.count().await.context("count after bulk delete")?;

    Ok(())
}

async fn connect_postgres() -> Result<(
    tokio_postgres::Client,
    tokio_postgres::Connection<tokio_postgres::Socket, tokio_postgres::tls::NoTlsStream>,
)> {
    let url = env::var("POSTGRES_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/flux_cookbook".to_string()
    });
    tokio_postgres::connect(&url, NoTls)
        .await
        .context("failed to connect to PostgreSQL")
}
