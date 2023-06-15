use std::{
    sync::Arc,
    time::{self, UNIX_EPOCH},
};

use key_rwlock::KeyRwLock;
use tokio::fs;
use tracing::{debug, error};
use uuid::Uuid;

use crate::config::Config;

/// Delete all programs that have not been used in a while.
pub async fn prune_programs(
    config: &Config,
    program_lock: Arc<KeyRwLock<Uuid>>,
) -> Result<(), std::io::Error> {
    debug!("pruning programs (ttl={})", config.program_ttl);

    let prune_until = time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - config.program_ttl;

    let mut pruned = 0;
    let mut it = fs::read_dir(&config.programs_dir).await?;
    while let Some(dir) = it.next_entry().await? {
        let Some(program_id) = dir
            .file_name()
            .to_str()
            .and_then(|x| x.parse::<Uuid>().ok())
        else {
            // always prune directories with names that cannot be parsed to uuids
            pruned += prune_directory(dir, prune_until).await as usize;
            continue;
        };

        // try to acquire the write lock without blocking
        if let Ok(_guard) = program_lock.try_write(program_id).await {
            pruned += prune_directory(dir, prune_until).await as usize;
            continue;
        }

        // if the write lock could not be acuired, spawn a task to wait for the lock
        tokio::spawn({
            let program_lock = Arc::clone(&program_lock);
            async move {
                let _guard = program_lock.write(program_id).await;
                if prune_directory(dir, prune_until).await {
                    debug!("successfully removed one old program");
                }
            }
        });
    }

    debug!("successfully removed {pruned} old program(s)");
    Ok(())
}

/// Check whether the program in a given directory has not been in use lately
/// and delete it in this case.
async fn prune_directory(dir: fs::DirEntry, prune_until: u64) -> bool {
    if !fs::try_exists(dir.path()).await.unwrap_or(false) {
        return false;
    }

    // read and check the timestamp of the program's last run
    match fs::read_to_string(dir.path().join("last_run"))
        .await
        .ok()
        .and_then(|lr| lr.parse::<u64>().ok())
    {
        Some(last_run) if last_run > prune_until => return false,
        _ => {}
    }

    let result = fs::remove_dir_all(dir.path()).await;
    match result {
        Ok(_) => true,
        Err(err) => {
            error!(
                "Failed to delete old program at {}: {err:#}",
                dir.path().display()
            );
            false
        }
    }
}
