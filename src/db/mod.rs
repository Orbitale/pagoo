use anyhow::Context;
use rusqlite::Connection;
use rusqlite::OpenFlags;
use rusqlite_migration::Migrations;
use rusqlite_migration::M;
use std::path::PathBuf;

pub(crate) fn get_database_connection(
    configured_path: Option<String>,
) -> anyhow::Result<Connection> {
    let database_path = get_database_path(configured_path);
    let database_flags = get_database_flags();

    let mut conn = Connection::open_with_flags(database_path, database_flags)?;

    let migrations = get_migrations();
    migrations.to_latest(&mut conn).unwrap();

    Ok(conn)
}

fn get_database_path(configured_path: Option<String>) -> PathBuf {
    let path = if let Some(file) = configured_path {
        file
    } else {
        let home_dir = crate::config::pagoo_home_dir();
        let home_dir = home_dir
            .to_str()
            .context("Could not get HOME dir.")
            .unwrap();
        format!("{}/data.db3", home_dir)
    };

    PathBuf::from(path)
}

fn get_database_flags() -> OpenFlags {
    let mut db_flags = OpenFlags::empty();

    db_flags.insert(OpenFlags::SQLITE_OPEN_READ_WRITE);
    db_flags.insert(OpenFlags::SQLITE_OPEN_CREATE);
    db_flags.insert(OpenFlags::SQLITE_OPEN_FULL_MUTEX);
    db_flags.insert(OpenFlags::SQLITE_OPEN_NOFOLLOW);
    db_flags.insert(OpenFlags::SQLITE_OPEN_PRIVATE_CACHE);

    db_flags
}

fn get_migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up(include_str!("./migrations/00-schema.sql"))])
}
