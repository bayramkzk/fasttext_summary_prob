// @generated automatically by Diesel CLI.

diesel::table! {
    messages (id) {
        id -> Int4,
        filename -> Text,
        message -> Text,
    }
}

diesel::table! {
    summary_probs (id) {
        id -> Int4,
        message_id -> Int4,
        lang -> Text,
        prob -> Float4,
    }
}

diesel::joinable!(summary_probs -> messages (message_id));

diesel::allow_tables_to_appear_in_same_query!(
    messages,
    summary_probs,
);
