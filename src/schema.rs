// @generated automatically by Diesel CLI.

diesel::table! {
    solana_transactions (id) {
        id -> Integer,
        #[max_length = 200]
        blockhash -> Varchar,
        last_valid_block_height -> Unsigned<Bigint>,
        status -> Unsigned<Smallint>,
        #[max_length = 2000]
        tx -> Varchar,
        created_at -> Datetime,
        updated_at -> Nullable<Datetime>,
        sent_at -> Nullable<Datetime>,
        confirmed_at -> Nullable<Datetime>,
        finalized_at -> Nullable<Datetime>,
        time_to_send -> Nullable<Unsigned<Integer>>,
        time_to_confirmed -> Nullable<Unsigned<Integer>>,
        time_to_finalized -> Nullable<Unsigned<Integer>>,
        priority_fee -> Nullable<Unsigned<Integer>>,
        #[max_length = 200]
        tx_signature -> Nullable<Varchar>,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        #[max_length = 255]
        name -> Varchar,
        age -> Nullable<Integer>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    solana_transactions,
    users,
);
