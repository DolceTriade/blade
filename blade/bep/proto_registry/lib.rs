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
    for (_, v) in &hs {
        let desc = fs::read(v).expect("failed to read descriptor");
        fds.merge(&desc[..]).expect("failed to merge descriptor");
    }
    fds
}

pub fn init_global_descriptor_pool() -> Result<()> {
    let b = &*DESCRIPTORS.encode_to_vec();
    DescriptorPool::decode_global_file_descriptor_set(b)
        .context("failed to load global descriptor pool")?;

    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() {
        assert_ne!(DESCRIPTORS.file.len(), 0);
    }

    #[test]
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
    }

    #[test]
    fn test_init_reflect_global_pool() {
        init_global_descriptor_pool().expect("failed to load descriptors into global pool");
        let global = DescriptorPool::global();
        let mut num_wkt = 0;
        let mut num_bep = 0;
        global.all_messages().for_each(|m| {
            println!("{}", m.full_name());
            if m.full_name().starts_with("google.protobuf.") {
                num_wkt += 1;
            } else {
                num_bep += 1;
            }
        });
        assert!(num_wkt > 0);
        assert!(num_bep > 0);
    }
}
