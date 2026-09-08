ALTER TABLE users
    ADD COLUMN email_canonical TEXT,
    ADD COLUMN password_hash TEXT NOT NULL DEFAULT '!unusable!',
    ADD COLUMN email_verified_at TIMESTAMPTZ;

UPDATE users
SET email_canonical = LOWER(BTRIM(email));

ALTER TABLE users
    ALTER COLUMN email_canonical SET NOT NULL,
    ALTER COLUMN password_hash DROP DEFAULT;

ALTER TABLE users DROP CONSTRAINT users_email_key;
ALTER TABLE users ADD CONSTRAINT users_email_canonical_key UNIQUE (email_canonical);

CREATE TABLE email_verification_tokens (
    user_id BIGINT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    token_hash BYTEA NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE password_reset_tokens (
    user_id BIGINT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    token_hash BYTEA NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
