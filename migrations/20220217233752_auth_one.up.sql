-- Add up migration script here
-- user_id would be coming from the request (but generate for now)

CREATE DATABASE auth_one(
    id UUID PRIMARY KEY UNIQUE DEFAULT uuid_generate_v4(),
    user_id UUID PRIMARY KEY UNIQUE DEFAULT uuid_generate_v4(),
    oauth_token VARCHAR(128) NOT NULL,
    oauth_secret VARCHAR(128) NOT NULL,
    oauth_verifier VARCHAR(128) NOT NULL
);