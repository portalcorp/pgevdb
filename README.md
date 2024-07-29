# PGEVDb: Embedded PostgreSQL Vector Database in Rust

[![Rust](https://img.shields.io/badge/rust-1.79%2B-blue.svg)](https://www.rust-lang.org/) [![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)

PGEVDb (PostgreSQL Embedded Vector Database) is a toy project that demonstrates how to set up and use a fully embedded PostgreSQL database with vector capabilities in Rust. The PostgreSQL database is embedded locally as one portable folder (at `./data/` relative to the project root or executable location) which has a starting size of around ~300mb.

First-time install time is ~30 seconds and subsequent startup times are <1s.

## 🚀 Features

- Embedded PostgreSQL database
- Automatic setup and configuration
- Integration of the [`pgvecto.rs`](https://github.com/tensorchord/pgvecto.rs/) extension
- Vector operations and similarity search
- Asynchronous Rust implementation

## 📋 Prerequisites

- Rust 1.75 or higher
- Cargo (Rust's package manager)

## 🛠 Installation

Clone the repository:

```
git clone https://github.com/yourusername/pgevdb.git
cd pgevdb
```

## 🏃‍♂️ Running the Project

Compile and run the binary:

```
cargo run
```

This will:

1. Set up an embedded PostgreSQL instance
2. Install and configure the `pg_vectors` extension
3. Create a sample table with vector data
4. Demonstrate vector operations and similarity search

## 🧰 How It Works

PGEVDb leverages several key components:

1. **Embedded PostgreSQL**: Uses the `postgresql_embedded` crate to run a full PostgreSQL instance within the Rust application.
2. **pg_vectors Extension**: Automatically downloads, installs, and configures the `pg_vectors` extension, enabling advanced vector operations.
3. **SQLx**: Employs the `sqlx` crate for type-safe SQL queries and database interactions.
4. **Tokio**: Utilizes the `tokio` runtime for asynchronous operations.

## 📊 Example Outputs

When you run the project, you'll see outputs demonstrating:

1. Vector distance calculations
2. Similarity searches
3. Basic CRUD operations with vector data

## 🤝 Contributing

Contributions, issues, and feature requests are welcome! Feel free to check [issues page](https://github.com/yourusername/pgevdb/issues).

## 📜 License

This project is [MIT](https://opensource.org/licenses/MIT) licensed.

## 🙏 Acknowledgements

- [postgresql_embedded](https://github.com/theseus-rs/postgresql-embedded)
- [pgvecto.rs](https://github.com/tensorchord/pgvecto.rs)
- [SQLx](https://github.com/launchbadge/sqlx)
- [Tokio](https://tokio.rs/)

Happy vector computing! 🚀🔢
