use std::{env, sync::Arc};

use anyhow::{Context, Result};
use flux::{Entity, ReadRepository, RepositoryError, WriteRepository};
use flux_postgres::{PostgresRepository, SqlEntity};
use tokio_postgres::{types::ToSql, NoTls, Row};

#[derive(Clone, Debug)]
struct GeneratedProduct {
    product_id: i64,
    name: String,
    price_cents: i32,
}

impl Entity for GeneratedProduct {
    type Id = i64;

    fn id(&self) -> &Self::Id {
        &self.product_id
    }

    fn has_id(&self) -> bool {
        self.product_id != 0
    }

    fn set_id(&mut self, id: Self::Id) {
        self.product_id = id;
    }
}

impl SqlEntity for GeneratedProduct {
    fn table_name() -> &'static str {
        "cookbook_generated_products"
    }

    fn primary_key() -> &'static str {
        "product_id"
    }

    fn fields() -> &'static [&'static str] {
        &["product_id", "name", "price_cents"]
    }

    fn from_row(row: Row) -> flux::Result<Self> {
        Ok(Self {
            product_id: row
                .try_get("product_id")
                .map_err(|error| RepositoryError::Backend(error.to_string()))?,
            name: row
                .try_get("name")
                .map_err(|error| RepositoryError::Backend(error.to_string()))?,
            price_cents: row
                .try_get("price_cents")
                .map_err(|error| RepositoryError::Backend(error.to_string()))?,
        })
    }

    fn to_insert_params(&self) -> Vec<&(dyn ToSql + Sync)> {
        vec![&self.product_id, &self.name, &self.price_cents]
    }

    fn to_update_params(&self) -> Vec<&(dyn ToSql + Sync)> {
        vec![&self.name, &self.price_cents]
    }

    fn primary_key_param(&self) -> &(dyn ToSql + Sync) {
        &self.product_id
    }
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
            CREATE TABLE IF NOT EXISTS cookbook_generated_products (
                product_id BIGSERIAL PRIMARY KEY,
                name TEXT NOT NULL,
                price_cents INTEGER NOT NULL
            );
            "#,
        )
        .await
        .context("failed to create cookbook_generated_products")?;

    let repo = PostgresRepository::<GeneratedProduct>::new(Arc::new(client));
    let unsaved = GeneratedProduct {
        product_id: 0,
        name: "Generated ID product".to_string(),
        price_cents: 4_200,
    };

    assert!(!unsaved.has_id());

    let saved = repo
        .insert(&unsaved)
        .await
        .context("insert generated product")?;
    assert!(saved.has_id());

    let loaded = repo
        .find_by_id(saved.id())
        .await
        .context("find generated product")?;
    assert_eq!(loaded.product_id, saved.product_id);

    repo.delete(saved.id())
        .await
        .context("delete generated product")?;

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
