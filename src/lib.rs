use clap::Parser;
use crc::{Crc, CRC_32_ISCSI};
use log::{debug, error, info, warn};
use rusqlite::{params, Connection, Error};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[clap(name = "multi-machine-dedup")]
pub struct CLI {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Parser, Debug)]
pub enum SubCommand {
    #[structopt(name = "index", about = "Use index to create or update a database")]
    Index(IndexOptions),
    #[structopt(
        name = "check-integrity",
        about = "Use check-integrity to verify all files"
    )]
    CheckIntegrity(CheckIntegrityOptions),
    #[structopt(name = "compare", about = "Use compare to compare two databases")]
    Compare(CompareOptions),
}

#[derive(Parser, Debug)]
pub struct IndexOptions {
    #[structopt(short, long)]
    pub label: String,
    #[structopt(short, long, parse(from_os_str))]
    pub db: PathBuf,
    pub path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct CheckIntegrityOptions {
    #[clap(short, long)]
    pub label: String,
    #[clap(short, long, parse(from_os_str))]
    pub db: PathBuf,
}

#[derive(Parser, Debug)]
pub struct CompareOptions {
    #[clap(long, parse(from_os_str))]
    pub db1: PathBuf,
    #[clap(long, parse(from_os_str))]
    pub db2: PathBuf,
}

#[derive(Clone, Debug)]
struct Entry {
    hash: u32,
    full_path: String,
    size: u64,
    mime: String,
}

pub const CASTAGNOLI: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);

pub fn create_db(conn: &Connection) {
    //id              INTEGER PRIMARY KEY AUTOINCREMENT,
    conn.execute(
        "CREATE TABLE IF NOT EXISTS file (
                  host TEXT,
                  full_path TEXT,
                  hash              INTEGER,
                  size INTEGER,              
                  PRIMARY KEY (host, full_path)
                  FOREIGN KEY(hash,size) REFERENCES hash(hash,size)
                  )",
        [],
    )
    .expect("");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS hash (
                  hash              INTEGER,
                  size INTEGER,
                  mime TEXT,
                  PRIMARY KEY (hash, size)
                  )",
        [],
    )
    .expect("");
}

pub fn index(opt: IndexOptions) {
    //let conn = Connection::open_in_memory().unwrap();
    let conn = Connection::open(opt.db).unwrap();
    create_db(&conn);
    for entry in WalkDir::new(opt.path) {
        let entry = entry.unwrap();
        if entry.file_type().is_dir() {
            info!("Processsing directory {}", entry.path().display());
        } else {
            let x = fs::metadata(entry.path()).unwrap().len();
            info!("Indexing file {} {:?}", entry.path().display(), x);
            let crc = hash(entry.path());
            debug!("The crc is: {} for file {}", crc, entry.path().display());
            let data = Entry {
                hash: crc,
                full_path: entry.path().display().to_string(),
                size: fs::metadata(entry.path()).unwrap().len(),
                mime: tree_magic_mini::from_filepath(entry.path())
                    .get_or_insert("N/A")
                    .to_string(),
            };
            let res = conn.execute(
                "INSERT INTO hash (hash, size, mime) VALUES (?1, ?2, ?3)",
                params![data.hash, data.size, data.mime],
            ); //.expect("req1");
            match res {
                Ok(_) => (),
                Err(error) => match error {
                    Error::SqliteFailure(error, _msg) => {
                        if error.extended_code == 1555 {
                            warn!(
                                "hash & size '{}' '{}' already indexed",
                                data.hash, data.size
                            )
                        }
                    }
                    _ => panic!(
                        "Unable to index hash & size: '{}' {}",
                        data.full_path, error
                    ),
                },
            }
            let res = conn.execute(
                "INSERT INTO file (host, full_path, hash, size) VALUES (?1, ?2, ?3, ?4)",
                params![opt.label, data.full_path, data.hash, data.size],
            ); //.expect("req2");
            match res {
                Ok(_) => (),
                Err(error) => match error {
                    Error::SqliteFailure(error, _msg) => {
                        if error.extended_code == 1555 {
                            error!("path '{}' already indexed", data.full_path)
                        }
                    }
                    _ => panic!("Unable to index file: '{}' {}", data.full_path, error),
                },
            }
        }
    }
}

pub fn check_integrity(opt: CheckIntegrityOptions) {
    let conn = Connection::open(opt.db).unwrap();
    let mut stmt = conn.prepare("SELECT * FROM file WHERE host=:host").unwrap();
    let file_iter = stmt
        .query_map(&[(":host", &opt.label)], |row| {
            Ok(Entry {
                hash: row.get(2)?,
                full_path: row.get(1)?,
                size: row.get(3)?,
                mime: "".to_string(),
            })
        })
        .unwrap();

    let mut ok_count = 0;
    let mut ko_count = 0;
    for file in file_iter {
        let stored_hash: u32 = file.as_ref().unwrap().hash;
        let path = &file.unwrap().full_path; //diff with &file.as_ref().unwrap() ??
        if stored_hash != hash(&PathBuf::from(path)) {
            ko_count += 1;
            error!("check failed on file: '{}'", path);
        } else {
            ok_count += 1;
            debug!("check ok on file: '{}'", path);
        }
    }
    if ko_count == 0 {
        info!("Integrity check OK, all {} files verified", ok_count);
    } else {
        error!("Integrity check failed, {} files are corrupted", ko_count);
    }
}

pub fn compare(opt: CompareOptions) {
    let conn1 = Connection::open(opt.db1).unwrap();
    let mut stmt1 = conn1.prepare("SELECT * FROM hash").unwrap();
    let conn2 = Connection::open(opt.db2).unwrap();
    let mut stmt2 = conn2.prepare("SELECT * FROM hash").unwrap();
    let file_iter1 = stmt1
        .query_map([], |row| {
            Ok(Entry {
                hash: row.get(0)?,
                full_path: "".to_string(),
                size: row.get(1)?,
                mime: "".to_string(),
            })
        })
        .unwrap();
    let mut count = 0;
    let mut entries = 0;
    for file1 in file_iter1 {
        entries += 1;
        let mut file_iter2 = stmt2
            .query_map([], |row| {
                Ok(Entry {
                    hash: row.get(0)?,
                    full_path: "".to_string(),
                    size: row.get(1)?,
                    mime: "".to_string(),
                })
            })
            .unwrap();
        let stored_hash1: u32 = file1.as_ref().unwrap().hash;
        let f = file_iter2.find(|h| h.as_ref().unwrap().hash == stored_hash1);
        if f.is_none() {
            warn!("hash {} not found in db2", stored_hash1);
            count += 1;
        }
    }

    if count == 0 {
        info!("All {} entries in db1 are also in db2", entries);
    } else {
        warn!("Missing/Mismatch {} entries", count);
    };
}

fn get_file_content(path: &Path) -> Vec<u8> {
    let mut buffer = BufReader::new(File::open(path).unwrap());
    let mut file_content = Vec::new();
    let _ = buffer.read_to_end(&mut file_content);
    file_content
}

fn hash(path: &Path) -> u32 {
    let mut digest = CASTAGNOLI.digest();
    let current_file_content = get_file_content(path);
    digest.update(&current_file_content);
    digest.finalize()
}
