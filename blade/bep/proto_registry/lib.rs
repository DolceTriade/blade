use anyhow::{Context, Result};
use lazy_static::lazy_static;
use prost::Message;
use prost_reflect::DescriptorPool;
use prost_types::FileDescriptorSet;
use runfiles::Runfiles;
use std::collections::HashMap;
use std::fs;
use walkdir::WalkDir;

lazy_static! {
    pub static ref DESCRIPTORS: Box<FileDescriptorSet> = Box::new(load());
}

fn load() -> FileDescriptorSet {
    let mut hs = HashMap::new();
    let r = Runfiles::create().expect("Must run using bazel with runfiles");
    let root = r.rlocation("");
    for entry in WalkDir::new(root).follow_links(true) {
        let p = entry.expect("invalid entry when walking runfiles");
        if p.path().to_string_lossy().ends_with("proto.bin") {
            hs.insert(
                p.path().file_name().unwrap().to_string_lossy().to_string(),
                p.path().to_path_buf(),
            );
        }
    }
    let mut fds: FileDescriptorSet = FileDescriptorSet::default();
    for v in hs.values() {
        let desc = fs::read(v).expect("failed to read descriptor");
        fds.merge(&desc[..]).expect("failed to merge descriptor");
    }
    fds
}

pub fn init_global_descriptor_pool() -> Result<()> {
    let b = &*DESCRIPTORS.encode_to_vec();
    DescriptorPool::decode_global_file_descriptor_set(b)
        .context("failed to load global descriptor pool")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use build_event_stream_proto::*;
    use prost_reflect::ReflectMessage;

    #[test]
    fn test_load() {
        assert_ne!(DESCRIPTORS.file.len(), 0);
    }

    #[test]
    #[should_panic]
    fn test_reflect_global_pool_default() {
        let global = DescriptorPool::global();
        let mut num_wkt = 0;
        let mut num_bep = 0;
        global.all_messages().for_each(|m| {
            if m.full_name().starts_with("google.protobuf.") {
                num_wkt += 1;
            } else {
                num_bep += 1;
            }
        });
        assert!(num_wkt > 0);
        assert_eq!(num_bep, 0);
        let be = build_event_stream::EnvironmentVariable {
            name: "PATH".into(),
            value: "/usr/bin".into(),
        };
        be.transcode_to_dynamic();
    }

    #[test]
    fn test_init_reflect_global_pool() {
        init_global_descriptor_pool().expect("failed to load descriptors into global pool");
        let global = DescriptorPool::global();
        let mut num_wkt = 0;
        let mut num_bep = 0;
        global.all_messages().for_each(|m| {
            if m.full_name().starts_with("google.protobuf.") {
                num_wkt += 1;
            } else {
                num_bep += 1;
            }
        });
        assert!(num_wkt > 0);
        assert!(num_bep > 0); 

        std::thread::sleep(std::time::Duration::from_secs(30));

        let be = build_event_stream::EnvironmentVariable {
            name: "PATH".into(),
            value: "/usr/bin".into(),
        };
        let d = be.transcode_to_dynamic();
        let j = serde_json::ser::to_string(&d).unwrap();
        assert_eq!(j, r#"{"name":"UEFUSA==","value":"L3Vzci9iaW4="}"#);
        let mut udo: std::path::PathBuf = std::env::var("TEST_UNDECLARED_OUTPUTS_DIR").unwrap().into();
        udo.push("random.txt");
        std::fs::write(udo, "random file").unwrap();
    }
}
