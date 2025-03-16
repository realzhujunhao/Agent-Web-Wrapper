use std::{env, fs::create_dir_all, path::PathBuf, sync::OnceLock};

use anyhow::Context;
use async_openai::{Client, config::OpenAIConfig};
use jwt_simple::prelude::HS256Key;
use sqlx::SqlitePool;

use crate::{
    CommandLineArgs, auth,
    config::{self, ServerConfig},
    indoc_info, store,
};

pub static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();
pub static SERVER_CONFIG: OnceLock<ServerConfig> = OnceLock::new();
pub static AGENT_CLIENT: OnceLock<Client<OpenAIConfig>> = OnceLock::new();
pub static DB_POOL: OnceLock<SqlitePool> = OnceLock::new();
pub static JWT_KEY: OnceLock<HS256Key> = OnceLock::new();
pub static COMMAND_LINE_ARGS: OnceLock<CommandLineArgs> = OnceLock::new();

// Set OnceLock value, panic in place with identifier.
// Logically it should never fail.
macro_rules! init_once {
    ($id:ident, $value:expr) => {
        $crate::indoc_debug!("State {} = {:?}", stringify!($id), $value);
        $id.set($value)
            .expect(concat!("init ", stringify!($id), " twice"))
    };
}

/// Set all global variables.
pub async fn init_states(cli: CommandLineArgs) -> anyhow::Result<()> {
    init_once!(COMMAND_LINE_ARGS, cli);

    // init directory
    let exec_path = env::current_exe().with_context(|| "exec path")?;
    let exec_path = exec_path
        .canonicalize()
        .with_context(|| "convert exec path to absolute path")?;
    let exec_dir = exec_path.parent().with_context(|| "exec has no parent")?;
    let data_dir = exec_dir.join("server_data");
    create_dir_all(&data_dir).with_context(|| "create data directory")?;

    init_once!(DATA_DIR, data_dir);

    // init config
    let (server_config, has_config) = config::init_config()?;
    if !has_config {
        indoc_info!("Config template generated, fill and restart.");
        std::process::exit(1);
    }

    // init client
    let openai_config = OpenAIConfig::new()
        .with_api_base(&server_config.api_base)
        .with_api_key(&server_config.api_key);
    let agent_client = Client::with_config(openai_config);

    // init db pool
    let pool = store::init_sqlite_pool(server_config.db_pool_size).await?;

    // init jwt key
    let jwt_key_bytes = auth::init_jwt_key()?;
    let jwt_key = HS256Key::from_bytes(&jwt_key_bytes);

    init_once!(AGENT_CLIENT, agent_client);
    init_once!(SERVER_CONFIG, server_config);
    init_once!(DB_POOL, pool);
    init_once!(JWT_KEY, jwt_key);
    Ok(())
}
