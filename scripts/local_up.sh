#!/bin/bash

set -e

docker_compose_content=$(cat <<EOF
version: "3"

services:
  postgres:
    image: "postgres:latest"
    environment:
      POSTGRES_USER: cuddle
      POSTGRES_PASSWORD: cuddle
      POSTGRES_DB: cuddle
    ports:
      - "5432:5432"
EOF)

docker-compose -p cuddle_local -f <(echo "$docker_compose_content") up -d --remove-orphans

cat <<EOF > local.env
DATABASE_URL=postgres://cuddle:cuddle@localhost:5432/cuddle
EOF

