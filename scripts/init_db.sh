#!/usr/bin/env bash

set -x 
set -eo pipefail

if ! [ -x "$(command -v psql)" ]; then
  echo 'Error: psql is not installed.' >&2
  exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 'Error: sqlx is not installed.'
  exit 1
fi

DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"

if [[ -z "${SKIP_DOCKER}" ]]; 
then
    docker run --rm \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        -p "${DB_PORT}":5432 \
        --name newsletter-db \
        -d postgres \
        postgres -N 1000
fi

export PGPASSWORD="${DB_PASSWORD}"
until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    echo "Postgres is still unavailable - sleeping" >&2 
    sleep 1
done

echo "Postgres is up and running on port ${DB_PORT}!" >&2

DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME} 
export DATABASE_URL

sqlx database create
sqlx migrate run
