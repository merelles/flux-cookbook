use std::{env, sync::Arc};

use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use flux::{GenericFilter, OrderDirection, PageRequest, ReadRepository, WriteRepository};
use flux_derive::{Entity, SqlEntity};
use flux_postgres::PostgresRepository;
use tokio_postgres::NoTls;
use uuid::Uuid;

#[derive(Clone, Debug, Entity, SqlEntity)]
#[table_name = "cookbook_events"]
struct Event {
    #[primary_key]
    event_id: Uuid,
    name: String,
    status: String,
    score: i32,
    created_at: chrono::DateTime<Utc>,
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
            CREATE TABLE IF NOT EXISTS cookbook_events (
                event_id UUID PRIMARY KEY,
                name TEXT NOT NULL,
                status TEXT NOT NULL,
                score INTEGER NOT NULL,
                created_at TIMESTAMPTZ NOT NULL
            );
            "#,
        )
        .await
        .context("failed to create cookbook_events")?;

    let repo = PostgresRepository::<Event>::new(Arc::new(client));
    let now = Utc::now();
    for (index, status) in ["open", "paid", "cancelled"].into_iter().enumerate() {
        let event = Event {
            event_id: Uuid::new_v4(),
            name: format!("event-{index}"),
            status: status.to_string(),
            score: 10 + index as i32,
            created_at: now - Duration::minutes(index as i64),
        };
        repo.save(&event).await.context("seed event")?;
    }

    let filter = GenericFilter::<Event>::new()
        .gte("created_at", now - Duration::hours(1))
        .gte("score", 10)
        .and_group(|query| {
            query
                .or(|query| query.eq("status", "open"))
                .or(|query| query.eq("status", "paid"))
        })
        .order_by("created_at", OrderDirection::Desc);

    let page = repo
        .find_page_with_filter(filter, PageRequest::cursor(10, None))
        .await
        .context("find filtered event page")?;

    assert!(!page.items.is_empty());
    assert!(page.items.iter().all(|event| event.status != "cancelled"));

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
