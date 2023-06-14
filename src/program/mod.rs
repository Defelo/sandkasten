use std::{ffi::OsString, future::Future, path::Path};

use tokio::fs;
use tracing::error;

use crate::sandbox::{Mount, MountType};

pub mod build;
pub mod prune;
pub mod run;

/// Create the [`Mount`]s for the given closure file.
async fn mounts_from_closure(closure: &Path) -> Result<Vec<Mount>, std::io::Error> {
    Ok(fs::read_to_string(closure)
        .await?
        .trim()
        .lines()
        .map(|line| Mount {
            dest: OsString::from(line).into(),
            typ: MountType::ReadOnly {
                src: OsString::from(line).into(),
            },
        })
        .collect())
}

/// Create a tempdir, run an async closure and delete the tempdir.
pub async fn with_tempdir<P, A>(
    path: P,
    closure: impl FnOnce(P) -> A,
) -> Result<A::Output, std::io::Error>
where
    P: AsRef<Path> + Clone + std::fmt::Debug,
    A: Future,
{
    fs::create_dir_all(&path).await?;
    let out = closure(path.clone()).await;
    if let Err(err) = fs::remove_dir_all(&path).await {
        error!("Failed to remove tempdir {path:?}: {err:#}");
    }
    Ok(out)
}
