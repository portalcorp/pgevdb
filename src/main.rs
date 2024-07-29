#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]

use anyhow::Result;
use postgresql_embedded::{PostgreSQL, Settings};
use semver::VersionReq;
use sqlx::postgres::PgConnection;
use sqlx::postgres::PgPool;
use sqlx::Executor;
use sqlx::Row;
use std::io::Cursor;
use std::path::PathBuf;

use tracing::info;

const PG_VERSION: &str = "16.3.0";
const DATABASE_NAME: &str = "test";

#[tokio::main]
async fn main() -> Result<()> {
    // Start tracing
    tracing_subscriber::fmt::init();

    let storage_dir: PathBuf = std::env::current_dir()?.join("data");

    let mut settings = Settings::default();
    // Generate a random password
    settings.password_file = storage_dir.join(".pgpass");
    if settings.password_file.exists() {
        settings.password = std::fs::read_to_string(settings.password_file.clone())?;
    }

    println!("Password file: {:?}", settings.password_file);
    println!("Password: {:?}", settings.password);

    let installation_dir = storage_dir.join("pg");
    let data_dir = storage_dir.join("pg_data");
    settings.installation_dir = installation_dir.clone();
    settings.data_dir = data_dir.clone();
    settings.temporary = false;

    settings.version = VersionReq::parse(format!("={}", PG_VERSION).as_str())?;

    info!("Starting PostgreSQL v{}", PG_VERSION);
    let mut postgresql = PostgreSQL::new(settings);
    postgresql.setup().await?;
    postgresql.start().await?;

    if !postgresql.database_exists(DATABASE_NAME).await? {
        info!("Creating database '{}'", DATABASE_NAME);
        postgresql.create_database(DATABASE_NAME).await?;
    }
    let database_url = postgresql.settings().url(DATABASE_NAME);

    let mut pool = PgPool::connect(database_url.as_str()).await?;

    info!("Checking if pg_vectors extension is installed");
    if !is_pg_vectors_extension_installed(&installation_dir).await? {
        info!("Installing pg_vectors extension");

        let mut conn: PgConnection = pool.acquire().await?.detach();
        install_pg_vectors_extension(&installation_dir).await?;
        configure_pg_vectors_extension(&mut conn).await?;
        info!("Successfully set up pg_vectors extension");

        // Restart PostgreSQL to apply changes and reconnect pool
        postgresql.stop().await?;
        postgresql.start().await?;
        pool.close().await;
        pool = PgPool::connect(database_url.as_str()).await?;

        info!("Enabling pg_vectors extension");
        enable_pg_vectors_extension(&pool).await?;
    }

    // Some tests to verify the extension is working

    println!("Creating table 'items' with vector column");
    create_table_items(&pool).await?;

    println!("Inserting vector data");
    insert_vector_data(&pool).await?;

    println!("Demonstrating vector operations");
    demonstrate_vector_operations(&pool).await?;

    println!("Searching for similar vectors");
    search_similar_vectors(&pool).await?;

    Ok(())
}

async fn is_pg_vectors_extension_installed(install_dir: &PathBuf) -> Result<bool> {
    Ok(install_dir
        .join(PG_VERSION)
        .join("lib")
        .join("vectors.so")
        .exists())
}

/// Downloads the pg_vectors extension from the GitHub release page and extracts it to the PostgreSQL installation directory.
async fn install_pg_vectors_extension(install_dir: &PathBuf) -> Result<()> {
    info!("Setting up PostgreSQL vector extension");

    // Download extension
    let url = "https://github.com/tensorchord/pgvecto.rs/releases/download/v0.3.0/vectors-pg16_x86_64-unknown-linux-gnu_0.3.0.zip";
    info!("Downloading extension from {}", url);
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;

    // Extract zip
    let target = PathBuf::from("vectors");
    info!("Extracting zip to {:?}", target);
    zip_extract::extract(Cursor::new(bytes), &target, false)?;

    // Get PostgreSQL directories
    let pg_dir = install_dir.join(PG_VERSION);
    let pkglibdir = pg_dir.join("lib").to_str().unwrap().to_string();
    let sharedir = pg_dir.join("share").to_str().unwrap().to_string();
    let extension_dir = format!("{}/extension", sharedir);

    // Copy files
    info!("Copying library to {}", pkglibdir);
    std::fs::copy("vectors/vectors.so", format!("{}/vectors.so", pkglibdir))?;

    info!("Copying schema files to {}", extension_dir);
    // Copy all version-specific SQL files
    for entry in std::fs::read_dir("vectors")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("vectors--")
        {
            let file_name = path.file_name().unwrap();
            std::fs::copy(
                &path,
                format!("{}/{}", extension_dir, file_name.to_str().unwrap()),
            )?;
        }
    }
    std::fs::copy(
        "vectors/vectors.control",
        format!("{}/vectors.control", extension_dir),
    )?;

    // Delete the extracted vectors directory
    info!("Deleting extracted vectors directory");
    std::fs::remove_dir_all(target)?;

    info!("PostgreSQL vector extension install complete");

    Ok(())
}

async fn configure_pg_vectors_extension(conn: &mut PgConnection) -> Result<()> {
    // Add extension to shared_preload_libraries
    info!("Adding extension to shared_preload_libraries");
    conn.execute("ALTER SYSTEM SET shared_preload_libraries = \"vectors.so\"")
        .await?;

    // Add extension to search_path
    info!("Adding extension to search_path");
    conn.execute("ALTER SYSTEM SET search_path = \"$user\", public, vectors")
        .await?;

    Ok(())
}

async fn enable_pg_vectors_extension(pool: &PgPool) -> Result<()> {
    let query = "CREATE EXTENSION vectors;";
    sqlx::query(query).execute(pool).await?;
    Ok(())
}

async fn create_table_items(pool: &PgPool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS items (
            id bigserial PRIMARY KEY,
            embedding vector(3) NOT NULL
        );",
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_vector_data(pool: &PgPool) -> Result<()> {
    sqlx::query("INSERT INTO items (embedding) VALUES ('[1,2,3]'), ('[4,5,6]');")
        .execute(pool)
        .await?;

    sqlx::query(
        "INSERT INTO items (embedding) VALUES (ARRAY[1, 2, 3]::real[]), (ARRAY[4, 5, 6]::real[]);",
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn demonstrate_vector_operations(pool: &PgPool) -> Result<()> {
    let queries = [
        "SELECT '[1, 2, 3]'::vector <-> '[3, 2, 1]'::vector AS squared_euclidean_distance;",
        "SELECT '[1, 2, 3]'::vector <#> '[3, 2, 1]'::vector AS negative_dot_product;",
        "SELECT '[1, 2, 3]'::vector <=> '[3, 2, 1]'::vector AS cosine_distance;",
    ];

    for query in queries {
        let result: (f32,) = sqlx::query_as(query).fetch_one(pool).await?;
        println!("{}: {}", query, result.0);
    }

    Ok(())
}

async fn search_similar_vectors(pool: &PgPool) -> Result<()> {
    let query = "SELECT id, embedding::text FROM items ORDER BY embedding <-> '[3,2,1]' LIMIT 5;";
    let rows = sqlx::query(query).fetch_all(pool).await?;

    println!("Similar vectors:");
    for row in rows {
        let id: i64 = row.get("id");
        let embedding: String = row.get("embedding");
        println!("ID: {}, Embedding: {}", id, embedding);
    }

    Ok(())
}
