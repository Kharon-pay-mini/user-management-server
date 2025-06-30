// @generated automatically by Diesel CLI.

diesel::table! {
    otp (otp_id) {
        otp_id -> Uuid,
        otp_code -> Int4,
        user_id -> Text,
        created_at -> Timestamptz,
        expires_at -> Timestamptz,
    }
}

diesel::table! {
    user_bank_account (id) {
        id -> Uuid,
        user_id -> Text,
        #[max_length = 255]
        bank_name -> Varchar,
        #[max_length = 50]
        account_number -> Varchar,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    user_security_logs (log_id) {
        log_id -> Uuid,
        user_id -> Text,
        #[max_length = 50]
        ip_address -> Varchar,
        #[max_length = 50]
        city -> Varchar,
        #[max_length = 50]
        country -> Varchar,
        failed_login_attempts -> Int4,
        flagged_for_review -> Bool,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    user_wallet (id) {
        id -> Uuid,
        user_id -> Text,
        #[max_length = 100]
        wallet_address -> Nullable<Varchar>,
        #[max_length = 50]
        network_used_last -> Nullable<Varchar>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 20]
        phone -> Nullable<Varchar>,
        last_logged_in -> Nullable<Timestamptz>,
        verified -> Bool,
        #[max_length = 10]
        role -> Varchar,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(otp -> users (user_id));
diesel::joinable!(user_bank_account -> users (user_id));
diesel::joinable!(user_security_logs -> users (user_id));
diesel::joinable!(user_wallet -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    otp,
    user_bank_account,
    user_security_logs,
    user_wallet,
    users,
);
