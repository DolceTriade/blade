mod sqlite;

pub trait DB {
    fn upsert_invocation(&mut self, invocation: &state::InvocationResults) -> anyhow::Result<()>;
    fn upsert_target(&mut self, id: &str, target: &state::Target) -> anyhow::Result<()>;
    fn upsert_test(&mut self, id: &str, test: &state::Test) -> anyhow::Result<()>;
    fn upsert_test_run(&mut self, id: &str, run: &state::TestRun) -> anyhow::Result<()>;
}
