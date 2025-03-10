

pub async fn dump_profile() -> Result<Vec<u8>> {
    let tmp_path = tempfile::tempdir().map_err(|_| {
        BuildTempPathSnafu {
            path: std::env::temp_dir(),
        }
        .build()
    })?;

    let mut path_buf = PathBuf::from(tmp_path.path());
    path_buf.push("greptimedb.hprof");

    let path = path_buf
        .to_str()
        .ok_or_else(|| BuildTempPathSnafu { path: &path_buf }.build())?
        .to_string();

    let mut bytes = CString::new(path.as_str())
        .map_err(|_| BuildTempPathSnafu { path: &path_buf }.build())?
        .into_bytes_with_nul();

    {
        // #safety: we always expect a valid temp file path to write profiling data to.
        let ptr = bytes.as_mut_ptr() as *mut c_char;
        unsafe {
            tikv_jemalloc_ctl::raw::write(PROF_DUMP, ptr)
                .context(DumpProfileDataSnafu { path: path_buf })?
        }
    }

    let mut f = tokio::fs::File::open(path.as_str())
        .await
        .context(OpenTempFileSnafu { path: &path })?;
    let mut buf = vec![];
    let _ = f
        .read_to_end(&mut buf)
        .await
        .context(OpenTempFileSnafu { path })?;
    Ok(buf)
}
