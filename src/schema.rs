// @generated automatically by Diesel CLI.

diesel::table! {
    user (id) {
        id -> Unsigned<Bigint>,
        name -> Varchar,
        email -> Varchar,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}
