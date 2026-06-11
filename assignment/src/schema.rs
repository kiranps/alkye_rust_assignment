diesel::table! {
    users (id) {
        id -> Integer,
        username -> Text,
        password -> Text,
        role -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    tasks (id) {
        id -> Integer,
        title -> Text,
        description -> Nullable<Text>,
        created_by -> Integer,
        assigned_to -> Nullable<Integer>,
        status -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(users, tasks);
