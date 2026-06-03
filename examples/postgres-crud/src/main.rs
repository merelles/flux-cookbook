use std::{env, sync::Arc};

use anyhow::{Context, Result};
use flux::{ReadRepository, WriteRepository};
use flux_derive::{Entity, SqlEntity};
use flux_postgres::PostgresRepository;
use tokio_postgres::NoTls;
use uuid::Uuid;

#[derive(Clone, Debug, Entity, SqlEntity)]
#[table_name = "cookbook_products"]
struct Product {
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
            CREATE TABLE IF NOT EXISTS cookbook_products (
                product_id UUID PRIMARY KEY,
                name TEXT NOT NULL,
                price_cents INTEGER NOT NULL
            );
            "#,
        )
        .await
        .context("failed to create cookbook_products")?;

    let repo = PostgresRepository::<Product>::new(Arc::new(client));
    let product = Product {
        product_id: Uuid::new_v4(),
        name: "Keyboard".to_string(),
        price_cents: 12_000,
    };

    let inserted = repo.insert(&product).await.context("insert product")?;
    let loaded = repo
        .find_by_id(&inserted.product_id)
        .await
        .context("find inserted product")?;
    let updated = repo
        .update(&Product {
            price_cents: 13_500,
            ..loaded
        })
        .await
        .context("update product")?;
    let saved = repo.save(&updated).await.context("save product")?;
    let exists = repo
        .exists(&saved.product_id)
        .await
        .context("check product existence")?;
    let count = repo.count().await.context("count products")?;
    let deleted = repo
        .delete(&saved.product_id)
        .await
        .context("delete product")?;

    assert!(exists);
    assert!(count > 0);
    assert!(deleted);

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
