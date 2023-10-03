-- Add migration script here
CREATE TABLE outbox (
    id UUID NOT NULL,
    metadata JSONB NOT NULL,
    content BYTEA NOT NULL,
    inserted_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    state VARCHAR NOT NULL,
);
