// @generated automatically by Diesel CLI.

diesel::table! {
    Invocations (id) {
        id -> Text,
        status -> Text,
        start -> Text,
        output -> Text,
        command -> Text,
        pattern -> Nullable<Text>,
    }
}

diesel::table! {
    Targets (id) {
        id -> Text,
        invocation_id -> Text,
        name -> Text,
        status -> Text,
        kind -> Text,
        start -> Text,
        end -> Nullable<Text>,
    }
}

diesel::table! {
    TestArtifacts (id) {
        id -> Text,
        invocation_id -> Text,
        test_run_id -> Text,
        name -> Text,
        uri -> Text,
    }
}

diesel::table! {
    TestRuns (id) {
        id -> Text,
        invocation_id -> Text,
        test_id -> Text,
        run -> Integer,
        shard -> Integer,
        attempt -> Integer,
        status -> Text,
        details -> Text,
        duration_s -> Double,
    }
}

diesel::table! {
    Tests (id) {
        id -> Text,
        invocation_id -> Text,
        name -> Text,
        status -> Text,
        duration_s -> Nullable<Double>,
        end -> Text,
        num_runs -> Nullable<Integer>,
    }
}

diesel::joinable!(Targets -> Invocations (invocation_id));
diesel::joinable!(TestArtifacts -> Invocations (invocation_id));
diesel::joinable!(TestArtifacts -> TestRuns (test_run_id));
diesel::joinable!(TestRuns -> Invocations (invocation_id));
diesel::joinable!(TestRuns -> Tests (test_id));
diesel::joinable!(Tests -> Invocations (invocation_id));

diesel::allow_tables_to_appear_in_same_query!(
    Invocations,
    Targets,
    TestArtifacts,
    TestRuns,
    Tests,
);
