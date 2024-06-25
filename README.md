# README

## Introduction

This project is the backend for a notification system.

The goal is for someone to be able to run only a one-liner from everywhere, like:

```
curl -H "Authorization: ...." https://.../<user-id>/notify
```

To receive a notification on their own screen.

This could be helpful for DevOps Engineers/SRE/Platform Engineers for example that are running long job on a server and want to be notified when its finished;
or when troubleshooting a CICD Pipeline, etc

This backend interacts with NATS to manage the messaging system and with a PostgresQL database to perform the user management

## How to run it

1. Provide the environment variable `DATABASE_CONNECTION_STRING` (PostgreSQL database for the user management). Tested with Supabase.
2. `cargo run`    

## Unit Tests

To run the tests, they must be executed sequentially (test database impact), using

```
cargo test -- --test-threads=1
```

## Setup locally

### 1. Run the NATS docker container

(Note: in a new terminal)

Go to the folder with the nats docker configuration, and run:

```
docker run --rm --name my-nats-server \
  -p 4222:4222 \
  -p 8222:8222 \
  -p 6222:6222 \
  -v $(pwd)/nats-server.conf:/nats-server.conf \
  -v $(pwd)/operator.jwt:/operator.jwt \
        -v $(pwd)/jwt:/jwt \
  nats-yohan:0.0.1 \
  -c /nats-server.conf
```

### 2. Run the NATS Authorization server

(Note: in a new terminal)

1. Go to the folder with the nats_authorization_server
2. `export AUTHORIZATION_DB_CONNECTION_STRING="host=aws-0-ap-southeast-1.pooler.supabase.com user=postgres.something password=SOMETHING dbname=postgres"`
3. `cargo run`

### 3. Run the main part

(Note: in a new terminal)


1. `export CREDS_BASE_PATH=<path of the local creds base>`
    - Ex: `export CREDS_BASE_PATH="/Users/yohangouzerh/.local/share/nats/nsc/keys/creds"`
2. `export TEST_OPERATOR_NAME="ServerBackend"`
3. `export DATABASE_CONNECTION_STRING="host=aws-0-ap-southeast-1.pooler.supabase.com user=postgres.something password=SOMETHING dbname=postgres"`

### 4. (Optional) Create a user

- Can be done if lost API KEY value as well

1. Delete any existing user (Note: deleting from the database is not enough, as we need to delete the file as well)

`curl -X POST 'http://127.0.0.1:9090/user/7c278ecc-d624-45a0-aa87-9add7253b517/nsc/delete'`

2. Create an NSC user

`curl -X POST 'http://127.0.0.1:9090/user/7c278ecc-d624-45a0-aa87-9add7253b517/nsc/create'`

3. Generate API KEY, then store it somewhere

`curl -X POST 'http://127.0.0.1:9090/user/7c278ecc-d624-45a0-aa87-9add7253b517/api-keys/create'`

### 5. Listen to the sub

(Note: in a new terminal)

1. Copy the user creds file

Example: `cp /Users/yohangouzerh/.local/share/nats/nsc/keys/creds/ServerBackend/7c278ecc-d624-45a0-aa87-9add7253b517/user_01.creds .`

2. Listen to the sub

`nats -s localhost:4222 "--creds=7c278ecc-d624-45a0-aa87-9add7253b517_user.creds" sub "topic01"`

### 6. Send a message

(Note: in a new terminal)

1. `http POST localhost:9090/send/7c278ecc-d624-45a0-aa87-9add7253b517 "message=done" "Authorization: <api-key-value>"`

(Here, we use httpie tool, but can use curl)

2. Verify that the message *done* have well been received in the terminal that listen to the sub