// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Integer,
        #[max_length = 255]
        name -> Varchar,
        age -> Nullable<Integer>,
    }
}
