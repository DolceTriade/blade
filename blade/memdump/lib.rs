use anyhow::{Context, anyhow};
use std::ffi::{CString, c_char};
use std::path::PathBuf;
use tokio::io::AsyncReadExt;

#[cfg(test)]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

const PROF_DUMP: &[u8] = b"prof.dump\0";
const PROF_ACTIVE: &[u8] = b"prof.active\0";

pub fn is_profiling_active() -> bool {
    unsafe {
        let Ok(e ) = tikv_jemalloc_ctl::raw::read(PROF_ACTIVE) else {
            return false;
        };
        return e;
    }
}

pub async fn dump_profile() -> anyhow::Result<Vec<u8>> {
    if !tikv_jemalloc_ctl::profiling::prof::read()? || !is_profiling_active() {
        return Err(anyhow!("profiling not enabled!"));
    }
    let tmp_path = tempdir::TempDir::new("blade-memdump").context("failed to create tempdir")?;

    let mut path_buf = PathBuf::from(tmp_path.path());
    path_buf.push("blade.hprof");

    let path = path_buf
        .to_str()
        .ok_or_else(|| anyhow!("failed to convert path to str"))?
        .to_string();

    let mut bytes = CString::new(path.as_str())
        .context(format!("failed to convert '{path:#?}' to bytes"))?
        .into_bytes_with_nul();

    {
        // #safety: we always expect a valid temp file path to write profiling data to.
        let ptr = bytes.as_mut_ptr() as *mut c_char;
        unsafe {
            tikv_jemalloc_ctl::raw::write(PROF_DUMP, ptr)
                .map_err(|e| anyhow!("failed to take profile: {e:#?}"))?
        }
    }

    let mut f = tokio::fs::File::open(path.as_str())
        .await
        .context("failed to open profile")?;
    let mut buf = vec![];
    let _ = f
        .read_to_end(&mut buf)
        .await
        .context("failed to read profile")?;
    Ok(buf)
}

pub async fn stats() -> anyhow::Result<Vec<u8>> {
    let mut opts = tikv_jemalloc_ctl::stats_print::Options::default();
    opts.json_format = true;
    let mut buf = Vec::<u8>::new();
    tikv_jemalloc_ctl::stats_print::stats_print(&mut buf, opts).context("failed to print stats")?;
    Ok(buf)
}

pub async fn enable_profiling(enable: bool) -> anyhow::Result<()> {
    if !tikv_jemalloc_ctl::profiling::prof::read()? {
        return Err(anyhow!("profiling not enabled!"));
    }
    unsafe {
        _ = tikv_jemalloc_ctl::raw::update(PROF_ACTIVE, enable).context("failed to set profiling status")?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::*;

    fn is_profiling_enabled() -> bool {
        let Ok(s) = tikv_jemalloc_ctl::profiling::prof::read() else {
            return false;
        };

        s
    }

    #[tokio::test]
    async fn test_stats() {
        let s = stats().await.unwrap();
        assert!(s.len() > 0);
    }

    #[tokio::test]
    async fn test_enabled() {
        if !is_profiling_enabled() {
            println!("Memory profiling disabled, skipping...");
            return;
        }
        enable_profiling(false).await.unwrap();

        dump_profile().await.unwrap();

        enable_profiling(true).await.unwrap();

        dump_profile().await.unwrap();
    }

    #[tokio::test]
    async fn test_disabled() {
        if is_profiling_enabled() {
            println!("Memory profiling enabled, skipping...");
            return;
        }
        enable_profiling(false).await.unwrap_err();

        dump_profile().await.unwrap_err();

        enable_profiling(true).await.unwrap_err();

        dump_profile().await.unwrap_err();
    }

}
