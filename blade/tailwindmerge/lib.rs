use std::collections::BTreeMap;

pub fn tailwind_merge(orig: &str, add: &str) -> String {
    let mut m = BTreeMap::new();

    orig.split_ascii_whitespace().for_each(|f| {
        f.rsplit_once('-')
            .map(|(prefix, _)| {
                m.insert(prefix, f);
            })
            .or_else(|| {
                m.insert(f, f);
                Some(())
            });
    });
    add.split_ascii_whitespace().for_each(|f| {
        f.rsplit_once('-')
            .map(|(prefix, _)| {
                m.insert(prefix, f);
            })
            .or_else(|| {
                m.insert(f, f);
                Some(())
            });
    });

    m.into_values()
        .fold("".to_string(), |mut acc, e| {
            acc += e;
            acc += " ";
            acc
        })
        .trim_end()
        .into()
}

#[cfg(test)]
mod tests {
    use crate::tailwind_merge;

    #[test]
    fn test_basic() {
        assert_eq!(tailwind_merge("m-1 m-2", ""), "m-2");
        assert_eq!(tailwind_merge("m-1", "m-2"), "m-2");
        assert_eq!(tailwind_merge("m-2", "m-1"), "m-1");
    }
}
