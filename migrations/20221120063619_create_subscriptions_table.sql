-- Add migration script here
CREATE TABLE subscriptions (
    id uuid NOT NULL,
    PRIMARY KEY (id),
    email TEXT not null unique,
    name text not null,
    subscribed_at timestamptz not null
)