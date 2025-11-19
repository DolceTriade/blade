// @generated automatically by Diesel CLI.

diesel::table! {
    invocations (id) {
        id -> Text,
        status -> Text,
        start -> Timestamptz,
        end -> Nullable<Timestamptz>,
        command -> Text,
        pattern -> Nullable<Text>,
        last_heartbeat -> Nullable<Timestamptz>,
        profile_uri -> Nullable<Text>,
    }
}

diesel::table! {
    options (id) {
        id -> Text,
        invocation_id -> Text,
        kind -> Text,
        keyval -> Text,
    }
}

diesel::table! {
    targets (id) {
        id -> Text,
        invocation_id -> Text,
        name -> Text,
        status -> Text,
        kind -> Text,
        start -> Timestamptz,
        end -> Nullable<Timestamptz>,
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
        end -> Timestamptz,
        num_runs -> Nullable<Int4>,
    }
}

diesel::table! {
    invocationoutput (id) {
        id -> Int4,
        invocation_id -> Text,
        line -> Text,
    }
}

diesel::table! {
    unique_test_names (name) {
        name -> Text,
    }
}

diesel::joinable!(options -> invocations (invocation_id));
diesel::joinable!(targets -> invocations (invocation_id));
diesel::joinable!(testartifacts -> invocations (invocation_id));
diesel::joinable!(testartifacts -> testruns (test_run_id));
diesel::joinable!(testruns -> invocations (invocation_id));
diesel::joinable!(testruns -> tests (test_id));
diesel::joinable!(tests -> invocations (invocation_id));
diesel::joinable!(invocationoutput -> invocations (invocation_id));

diesel::allow_tables_to_appear_in_same_query!(
    invocations,
    invocationoutput,
    options,
    targets,
    testartifacts,
    testruns,
    tests,
    unique_test_names,
);
