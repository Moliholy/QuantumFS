use std::error::Error;

use rusqlite::{Connection, NO_PARAMS, ToSql};

use crate::errors::QFSError;
use crate::models::catalog::CatalogReference;
use crate::models::directoryentry::{self, DirectoryEntry};
use crate::types::ipfs::IpfsHash;

lazy_static! {
    static ref LISTING_QUERY: String = format!(
    "SELECT {} \
        FROM catalog \
        WHERE parent = ? \
        ORDER BY name ASC;", directoryentry::DATABASE_FIELDS
    );

    static ref INSERT_QUERY: String = format!(
    "INSERT INTO catalog ({}) \
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)", directoryentry::DATABASE_FIELDS
    );

    static ref FIND_PATH: String = format!(
    "SELECT {} \
        FROM catalog \
        WHERE path = ? \
        ORDER BY name ASC \
        LIMIT 1;", directoryentry::DATABASE_FIELDS
    );

    static ref LIST_NESTED: String = String::from(
    "SELECT path, hash, size \
        FROM nested_catalogs;"
    );
    static ref CREATE_CATALOG: String = String::from(
    "CREATE TABLE catalog
        (path TEXT, parent TEXT,\
        hardlinks INTEGER, hash BLOB, size INTEGER, mode INTEGER, mtime INTEGER,\
        flags INTEGER, name TEXT, symlink TEXT, uid INTEGER, gid INTEGER, \
        xattr BLOB, \
        CONSTRAINT pk_catalog PRIMARY KEY (path));"
    );

    static ref CREATE_INDEX: String = String::from(
    "CREATE INDEX idx_catalog_parent \
        ON catalog (parent);"
    );

    static ref CREATE_NESTED_CATALOGS: String = String::from(
    "CREATE TABLE nested_catalogs (path TEXT, hash TEXT, size INTEGER, \
        CONSTRAINT pk_nested_catalogs PRIMARY KEY (path));"
    );
}


pub fn add_directory_entry(connection: &Connection, dirent: &DirectoryEntry) -> Result<(), QFSError> {
    let mut statement = connection
        .prepare(INSERT_QUERY.as_str())
        .unwrap();
    let result = statement.insert(&[
        &dirent.path.to_string() as &dyn ToSql,
        &dirent.parent.to_string() as &dyn ToSql,
        &dirent.hash.to_string() as &dyn ToSql,
        &dirent.flags,
        &dirent.size,
        &dirent.mode,
        &dirent.mtime,
        &dirent.name,
        &dirent.symlink,
    ]);
    match result {
        Ok(num_rows) => if num_rows == 1 { Ok(()) } else { Err(QFSError::new("Error adding a directory entry")) },
        Err(error) => Err(QFSError::new(error.description()))
    }
}

pub fn create_catalog(connection: &Connection) -> Result<(), QFSError> {
    connection.execute_batch(
        format!(
            "BEGIN; \
                {}; \
                {}; \
                {}; \
            COMMIT;",
            CREATE_CATALOG.as_str(), CREATE_INDEX.as_str(), CREATE_NESTED_CATALOGS.as_str()
        ).as_str()
    ).map_err(QFSError::from)
}

pub fn list_nested(connection: &Connection) -> Result<Vec<CatalogReference>, QFSError> {
    let mut statement = connection
        .prepare(LIST_NESTED.as_str())
        .unwrap();
    let mut rows = statement.query(NO_PARAMS)?;
    let mut nested = Vec::new();
    while let Ok(Some(row)) = rows.next() {
        let path: String = row.get(0).unwrap();
        let hash: String = row.get(1).unwrap();
        let catalog_reference = CatalogReference::new(
            &IpfsHash::new(path.as_str()).unwrap(),
            &IpfsHash::new(hash.as_str()).unwrap(),
            row.get(3).unwrap(),
        );
        nested.push(catalog_reference);
    }
    Ok(nested)
}

pub fn find_directory_entry(connection: &Connection, hashed_path: String) -> Result<DirectoryEntry, QFSError> {
    let mut statement = connection
        .prepare(FIND_PATH.as_str())
        .unwrap();

    let mut rows = statement.query(&[hashed_path])?;
    if let Ok(Some(row)) = rows.next() {
        let dirent = DirectoryEntry::from_sql_row(row);
        return Ok(dirent);
    }
    Err(QFSError::new("Entry not found"))
}

pub fn list_directory(connection: &Connection, hashed_path: String) -> Result<Vec<DirectoryEntry>, QFSError> {
    let mut statement = connection
        .prepare(LISTING_QUERY.as_str())
        .unwrap();
    let mut rows = statement.query(&[hashed_path])?;
    let mut dirents = Vec::new();
    while let Ok(Some(row)) = rows.next() {
        dirents.push(DirectoryEntry::from_sql_row(row));
    }
    Ok(dirents)
}
