// @generated automatically by Diesel CLI.

diesel::table! {
    invocations (id) {
        id -> Text,
        status -> Text,
        start -> Text,
        output -> Text,
        command -> Text,
        pattern -> Nullable<Text>,
    }
}

diesel::table! {
    targets (id) {
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
    testartifacts (id) {
        id -> Text,
        invocation_id -> Text,
        test_run_id -> Text,
        name -> Text,
        uri -> Text,
    }
}

diesel::table! {
    testruns (id) {
        id -> Text,
        invocation_id -> Text,
        test_id -> Text,
        run -> Int4,
        shard -> Int4,
        attempt -> Int4,
        status -> Text,
        details -> Text,
        duration_s -> Float8,
    }
}

diesel::table! {
    tests (id) {
        id -> Text,
        invocation_id -> Text,
        name -> Text,
        status -> Text,
        duration_s -> Nullable<Float8>,
        num_runs -> Nullable<Int4>,
    }
}

diesel::joinable!(targets -> invocations (invocation_id));
diesel::joinable!(testartifacts -> invocations (invocation_id));
diesel::joinable!(testartifacts -> testruns (test_run_id));
diesel::joinable!(testruns -> invocations (invocation_id));
diesel::joinable!(testruns -> tests (test_id));
diesel::joinable!(tests -> invocations (invocation_id));

diesel::allow_tables_to_appear_in_same_query!(
    invocations,
    targets,
    testartifacts,
    testruns,
    tests,
);
