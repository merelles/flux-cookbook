use std::{env, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use flux::{Entity, ReadRepository, RepositoryError, WriteRepository};
use flux_mongodb::{MongoEntity, MongoObjectId, MongoRepository};
use flux_postgres::{PostgresRepository, SqlEntity};
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Client as MongoClient,
};
use tokio::time::sleep;
use tokio_postgres::{types::ToSql, NoTls, Row};

#[derive(Clone, Debug)]
struct MongoUser {
    id: MongoObjectId,
    name: String,
    email: String,
}

impl Entity for MongoUser {
    type Id = MongoObjectId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn has_id(&self) -> bool {
        true
    }
}

impl MongoEntity for MongoUser {
    fn collection_name() -> &'static str {
        "users"
    }

    fn from_document(document: Document) -> flux::Result<Self> {
        let id = document
            .get_object_id("_id")
            .map_err(|err| RepositoryError::Backend(err.to_string()))?;
        let name = document
            .get_str("name")
            .map_err(|err| RepositoryError::Backend(err.to_string()))?
            .to_string();
        let email = document
            .get_str("email")
            .map_err(|err| RepositoryError::Backend(err.to_string()))?
            .to_string();

        Ok(Self {
            id: MongoObjectId(id),
            name,
            email,
        })
    }

    fn to_document(&self) -> flux::Result<Document> {
        Ok(doc! {
            "_id": self.id.0,
            "name": &self.name,
            "email": &self.email,
        })
    }
}

#[derive(Clone, Debug)]
struct PostgresUser {
    user_id: String,
    name: String,
    email: String,
    source: String,
}

impl From<MongoUser> for PostgresUser {
    fn from(user: MongoUser) -> Self {
        Self {
            user_id: user.id.0.to_hex(),
            name: user.name,
            email: user.email,
            source: "mongodb".to_string(),
        }
    }
}

impl Entity for PostgresUser {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.user_id
    }

    fn has_id(&self) -> bool {
        !self.user_id.is_empty()
    }
}

impl SqlEntity for PostgresUser {
    fn table_name() -> &'static str {
        "cookbook_users"
    }

    fn primary_key() -> &'static str {
        "user_id"
    }

    fn fields() -> &'static [&'static str] {
        &["user_id", "name", "email", "source"]
    }

    fn from_row(row: Row) -> flux::Result<Self> {
        Ok(Self {
            user_id: row
                .try_get("user_id")
                .map_err(|err| RepositoryError::Backend(err.to_string()))?,
            name: row
                .try_get("name")
                .map_err(|err| RepositoryError::Backend(err.to_string()))?,
            email: row
                .try_get("email")
                .map_err(|err| RepositoryError::Backend(err.to_string()))?,
            source: row
                .try_get("source")
                .map_err(|err| RepositoryError::Backend(err.to_string()))?,
        })
    }

    fn to_insert_params(&self) -> Vec<&(dyn ToSql + Sync)> {
        vec![&self.user_id, &self.name, &self.email, &self.source]
    }

    fn to_update_params(&self) -> Vec<&(dyn ToSql + Sync)> {
        vec![&self.name, &self.email, &self.source]
    }

    fn primary_key_param(&self) -> &(dyn ToSql + Sync) {
        &self.user_id
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let config = CookbookConfig::from_env()?;
    let mongo_database = connect_mongo(&config).await?;
    let (postgres_client, connection) = connect_postgres(&config).await?;

    tokio::spawn(async move {
        if let Err(err) = connection.await {
            eprintln!("postgres connection error: {err}");
        }
    });

    ensure_postgres_schema(&postgres_client).await?;

    let mongo_repo = MongoRepository::<MongoUser>::new(mongo_database);
    let postgres_repo = PostgresRepository::<PostgresUser>::new(Arc::new(postgres_client));

    let user_id = MongoObjectId(config.mongo_user_id);

    if config.seed {
        let seed_user = MongoUser {
            id: user_id,
            name: config.seed_name.clone(),
            email: config.seed_email.clone(),
        };
        mongo_repo
            .save(&seed_user)
            .await
            .context("failed to seed Mongo user")?;
    }

    let mongo_user = mongo_repo
        .find_by_id(&user_id)
        .await
        .context("failed to read user from MongoDB")?;
    let postgres_user = PostgresUser::from(mongo_user);
    let saved_user = postgres_repo
        .save(&postgres_user)
        .await
        .context("failed to save user into PostgreSQL")?;

    println!(
        "copied Mongo user {} into Postgres table cookbook_users",
        saved_user.user_id
    );

    Ok(())
}

struct CookbookConfig {
    mongo_uri: String,
    mongo_database: String,
    mongo_user_id: ObjectId,
    postgres_url: String,
    seed: bool,
    seed_name: String,
    seed_email: String,
}

impl CookbookConfig {
    fn from_env() -> Result<Self> {
        let mongo_user_id = env_or("MONGO_USER_ID", "665f1b8b0fd3f6d8a8f54a01");

        Ok(Self {
            mongo_uri: env_or("MONGO_URI", "mongodb://localhost:27017"),
            mongo_database: env_or("MONGO_DATABASE", "flux_cookbook"),
            mongo_user_id: ObjectId::parse_str(&mongo_user_id)
                .with_context(|| format!("invalid MONGO_USER_ID: {mongo_user_id}"))?,
            postgres_url: env_or(
                "POSTGRES_URL",
                "postgres://postgres:postgres@localhost:5432/flux_cookbook",
            ),
            seed: env_or("COOKBOOK_SEED", "true").eq_ignore_ascii_case("true"),
            seed_name: env_or("COOKBOOK_USER_NAME", "Ada Lovelace"),
            seed_email: env_or("COOKBOOK_USER_EMAIL", "ada@example.com"),
        })
    }
}

async fn connect_mongo(config: &CookbookConfig) -> Result<mongodb::Database> {
    let client = retry("connect to MongoDB", || async {
        let client = MongoClient::with_uri_str(&config.mongo_uri).await?;
        client
            .database("admin")
            .run_command(doc! { "ping": 1 })
            .await?;
        Ok::<_, mongodb::error::Error>(client)
    })
    .await?;

    Ok(client.database(&config.mongo_database))
}

async fn connect_postgres(
    config: &CookbookConfig,
) -> Result<(
    tokio_postgres::Client,
    tokio_postgres::Connection<tokio_postgres::Socket, tokio_postgres::tls::NoTlsStream>,
)> {
    retry("connect to PostgreSQL", || async {
        tokio_postgres::connect(&config.postgres_url, NoTls).await
    })
    .await
    .context("failed to connect to PostgreSQL")
}

async fn ensure_postgres_schema(client: &tokio_postgres::Client) -> Result<()> {
    client
        .batch_execute(
            r#"
            CREATE TABLE IF NOT EXISTS cookbook_users (
                user_id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT NOT NULL,
                source TEXT NOT NULL
            );
            "#,
        )
        .await
        .context("failed to ensure PostgreSQL schema")
}

async fn retry<T, E, Fut, F>(label: &str, mut operation: F) -> Result<T>
where
    E: std::error::Error + Send + Sync + 'static,
    Fut: std::future::Future<Output = std::result::Result<T, E>>,
    F: FnMut() -> Fut,
{
    let attempts = env_or("COOKBOOK_CONNECT_ATTEMPTS", "30")
        .parse::<u32>()
        .unwrap_or(30);

    for attempt in 1..=attempts {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(err) if attempt == attempts => {
                return Err(err)
                    .with_context(|| format!("{label} failed after {attempts} attempts"));
            }
            Err(err) => {
                eprintln!("{label} attempt {attempt}/{attempts} failed: {err}");
                sleep(Duration::from_secs(2)).await;
            }
        }
    }

    unreachable!("retry loop always returns")
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}
