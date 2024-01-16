use time::OffsetDateTime;

pub fn to_systemtime(t: &OffsetDateTime) -> anyhow::Result<std::time::SystemTime> {
    prototime::timestamp::from_unix_sec(t.unix_timestamp() as u64)
}
