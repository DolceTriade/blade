mod duration {
    use anyhow::anyhow;
    use duration_proto::google::protobuf::Duration as DurationProto;
    use std::time::Duration;

    #[allow(dead_code)]
    pub fn from_proto(d: &DurationProto) -> anyhow::Result<std::time::Duration> {
        if d.seconds < 0 || d.nanos < 0 {
            return Err(anyhow!(
                "seconds and nanos cannot be negative: sec={} nanos={}",
                d.seconds,
                d.nanos
            ));
        }
        let nanos: u64 = d.nanos as u64;
        Ok(Duration::from_secs(d.seconds as u64) + Duration::from_nanos(nanos))
    }

    #[allow(dead_code)]
    pub fn to_proto(d: &std::time::Duration) -> DurationProto {
        let sec = d.as_secs();
        let nanos = d.subsec_nanos();

        DurationProto {
            seconds: sec as i64,
            nanos: nanos as i32,
        }
    }
}

mod timestamp {
    use anyhow::anyhow;
    use std::time::Duration;
    use std::time::UNIX_EPOCH;
    use timestamp_proto::google::protobuf::Timestamp as TimestampProto;

    #[allow(dead_code)]
    pub fn from_proto(t: &TimestampProto) -> anyhow::Result<std::time::SystemTime> {
        if t.seconds < 0 || t.nanos < 0 {
            return Err(anyhow!(
                "seconds and nanos cannot be negative: sec={} nanos={}",
                t.seconds,
                t.nanos
            ));
        }

        let nanos: u64 = t.nanos as u64;
        let d = Duration::from_secs(t.seconds as u64) + Duration::from_nanos(nanos);

        UNIX_EPOCH
            .checked_add(d)
            .ok_or_else(|| anyhow!("overflow when adding time"))
    }

    #[allow(dead_code)]
    pub fn to_proto(t: &std::time::SystemTime) -> anyhow::Result<TimestampProto> {
        let d = t.duration_since(UNIX_EPOCH)?;
        let sec = d.as_secs();
        let nanos = d.subsec_nanos();

        Ok(TimestampProto {
            seconds: sec as i64,
            nanos: nanos as i32,
        })
    }

    #[allow(dead_code)]
    pub fn to_unix_sec(t: &std::time::SystemTime) -> u64 {
        let Ok(d) = t.duration_since(UNIX_EPOCH) else {
            return 0;
        };
        d.as_secs()
    }

    #[allow(dead_code)]
    pub fn from_unix_sec(u: u64) -> anyhow::Result<std::time::SystemTime> {
        let d = Duration::from_secs(u);
        UNIX_EPOCH
            .checked_add(d)
            .ok_or_else(|| anyhow!("overflow when adding time"))
    }
}

#[cfg(test)]
mod test {
    use std::time::UNIX_EPOCH;

    use duration_proto::google::protobuf::Duration as DurationProto;
    use timestamp_proto::google::protobuf::Timestamp as TimestampProto;

    #[test]
    fn test_convert_duration() {
        let dp = DurationProto {
            seconds: 10,
            nanos: 1,
        };
        let d = super::duration::from_proto(&dp).unwrap();
        assert_eq!(d.as_secs(), dp.seconds as u64);
        assert_eq!(d.subsec_nanos(), 1);

        let new_dp = super::duration::to_proto(&d);
        assert_eq!(new_dp, dp);
    }

    #[test]
    fn test_bad_convert_duration() {
        let dp = DurationProto {
            seconds: -10,
            nanos: 1,
        };
        let _ = super::duration::from_proto(&dp).unwrap_err();
    }

    #[test]
    fn test_convert_timestamp() {
        let now = std::time::SystemTime::now();
        let now_proto = super::timestamp::to_proto(&now).unwrap();
        assert!(now_proto.seconds > 0);
        assert!(now_proto.nanos > 0);
        let new_now = super::timestamp::from_proto(&now_proto).unwrap();
        assert_eq!(now, new_now);
    }

    #[test]
    fn test_overflow_timestamp() {
        let t = TimestampProto {
            nanos: i32::MAX,
            seconds: i64::MAX,
        };
        let _ = super::timestamp::from_proto(&t).unwrap_err();
    }

    #[test]
    fn test_to_unix() {
        assert_eq!(super::timestamp::to_unix_sec(&UNIX_EPOCH), 0);
        assert_eq!(super::timestamp::from_unix_sec(0).unwrap(), UNIX_EPOCH);
        assert!(super::timestamp::to_unix_sec(&std::time::SystemTime::now()) > 0);
    }
}
