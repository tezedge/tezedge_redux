use rocksdb::DB;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use storage::{
    database::tezedge_database::{TezedgeDatabase, TezedgeDatabaseBackendConfiguration},
    initializer::{
        DbsRocksDbTableInitializer, GlobalRocksDbCacheHolder, MainChain, RocksDbCache,
        RocksDbColumnFactory, RocksDbConfig,
    },
    persistent::{
        database::open_kv, open_cl, open_main_db, sequence::Sequences, CommitLogSchema, DBError,
        DbConfiguration,
    },
    BlockStorage, PersistentStorage,
};

fn initialize_rocksdb<Factory: RocksDbColumnFactory>(
    config: &RocksDbConfig<Factory>,
) -> Result<Arc<DB>, DBError> {
    let kv_cache = RocksDbCache::new_lru_cache(config.cache_size)
        .expect("Failed to initialize RocksDB cache (db)");

    let db = open_kv(
        &config.db_path,
        config.columns.create(&kv_cache),
        &DbConfiguration {
            max_threads: config.threads,
        },
    )
    .map(Arc::new)?;

    Ok(db)
}

fn initialize_maindb<C: RocksDbColumnFactory>(
    kv: Option<Arc<DB>>,
    config: &RocksDbConfig<C>,
) -> Arc<TezedgeDatabase> {
    Arc::new(
        open_main_db(kv, config, TezedgeDatabaseBackendConfiguration::RocksDB)
            .expect("Failed to create/initialize MainDB database (db)"),
    )
}

pub fn init_storage() -> PersistentStorage {
    let config = RocksDbConfig {
        cache_size: 1024 * 1024,
        expected_db_version: 20,
        db_path: PathBuf::from("./data/db"),
        columns: DbsRocksDbTableInitializer,
        threads: Some(4),
    };

    let maindb = {
        let kv =
            initialize_rocksdb(&config).expect("Failed to create/initialize RocksDB database (db)");
        initialize_maindb(Some(kv), &config)
    };

    let commit_logs = Arc::new(
        open_cl(Path::new("./data"), vec![BlockStorage::descriptor()])
            .expect("Failed to open plain block_header storage"),
    );
    let sequences = Arc::new(Sequences::new(maindb.clone(), 1000));

    PersistentStorage::new(maindb, commit_logs, sequences)
}
