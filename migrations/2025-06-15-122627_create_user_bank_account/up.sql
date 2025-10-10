-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS user_bank_account (
    id UUID PRIMARY KEY DEFAULT (uuid_generate_v4()),
    user_id VARCHAR(50) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    bank_name VARCHAR(55) NOT NULL,
    account_number VARCHAR(50) NOT NULL,
    account_name VARCHAR(50) NOT NULL,
    phone VARCHAR(25) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);