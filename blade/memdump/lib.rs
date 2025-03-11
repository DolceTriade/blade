use anyhow::{Context, anyhow};
use std::ffi::{CString, c_char};
use std::path::PathBuf;
use tokio::io::AsyncReadExt;

const PROF_DUMP: &[u8] = b"prof.dump\0";

pub async fn dump_profile() -> anyhow::Result<Vec<u8>> {
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
