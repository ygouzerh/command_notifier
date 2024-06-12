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

## Tests

To run the tests, they must be executed sequentially (test database impact), using

```
cargo test -- --test-threads=1
```