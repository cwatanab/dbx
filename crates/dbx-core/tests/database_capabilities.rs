use dbx_core::database_capabilities::{
    agent_key, is_agent_type, is_metadata_connection_scoped, is_single_connection_pool, skips_tcp_probe,
};
use dbx_core::models::connection::DatabaseType;

#[test]
fn maps_agent_database_types_to_driver_keys() {
    assert_eq!(agent_key(&DatabaseType::Trino, None), Some("trino"));
    assert_eq!(agent_key(&DatabaseType::Hive, None), Some("hive"));
    assert_eq!(agent_key(&DatabaseType::Gaussdb, None), Some("gaussdb"));
    assert_eq!(agent_key(&DatabaseType::Tdengine, None), Some("tdengine"));
    assert_eq!(agent_key(&DatabaseType::Yashandb, None), Some("yashandb"));
    assert_eq!(agent_key(&DatabaseType::Access, None), Some("access"));
    assert_eq!(agent_key(&DatabaseType::Oracle, None), Some("oracle"));
    assert_eq!(agent_key(&DatabaseType::Oracle, Some("oracle-10g")), Some("oracle-10g"));
    assert_eq!(agent_key(&DatabaseType::Postgres, None), None);
}

#[test]
fn classifies_agent_database_types() {
    assert!(is_agent_type(&DatabaseType::Oracle));
    assert!(is_agent_type(&DatabaseType::Trino));
    assert!(is_agent_type(&DatabaseType::Hive));
    assert!(is_agent_type(&DatabaseType::Tdengine));
    assert!(is_agent_type(&DatabaseType::Yashandb));
    assert!(is_agent_type(&DatabaseType::Access));
    assert!(!is_agent_type(&DatabaseType::Mysql));
    assert!(!is_agent_type(&DatabaseType::Jdbc));
}

#[test]
fn identifies_single_connection_pool_types() {
    assert!(is_single_connection_pool(&DatabaseType::Sqlite));
    assert!(is_single_connection_pool(&DatabaseType::DuckDb));
    assert!(is_single_connection_pool(&DatabaseType::Oracle));
    assert!(is_single_connection_pool(&DatabaseType::Dameng));
    assert!(is_single_connection_pool(&DatabaseType::Access));
    assert!(is_single_connection_pool(&DatabaseType::Yashandb));
    assert!(is_single_connection_pool(&DatabaseType::Jdbc));
    assert!(!is_single_connection_pool(&DatabaseType::Trino));
    assert!(!is_single_connection_pool(&DatabaseType::Postgres));
}

#[test]
fn identifies_metadata_connections_that_drop_database_scope() {
    assert!(is_metadata_connection_scoped(&DatabaseType::Mysql));
    assert!(is_metadata_connection_scoped(&DatabaseType::Doris));
    assert!(is_metadata_connection_scoped(&DatabaseType::StarRocks));
    assert!(!is_metadata_connection_scoped(&DatabaseType::Postgres));
    assert!(!is_metadata_connection_scoped(&DatabaseType::Oracle));
}

#[test]
fn skips_tcp_probe_for_local_file_plugin_and_agent_types() {
    assert!(skips_tcp_probe(&DatabaseType::Sqlite));
    assert!(skips_tcp_probe(&DatabaseType::DuckDb));
    assert!(skips_tcp_probe(&DatabaseType::Jdbc));
    assert!(skips_tcp_probe(&DatabaseType::Access));
    assert!(skips_tcp_probe(&DatabaseType::Trino));
    assert!(skips_tcp_probe(&DatabaseType::Oracle));
    assert!(skips_tcp_probe(&DatabaseType::Tdengine));
    assert!(skips_tcp_probe(&DatabaseType::Yashandb));
    assert!(!skips_tcp_probe(&DatabaseType::Postgres));
    assert!(!skips_tcp_probe(&DatabaseType::Mysql));
}
