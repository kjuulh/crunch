# Crunch - Domain event interface

Crunch allows services to distribute their own events, as domain events. Domain events are a more strict representation on the api between services in different domains, they are meant to tell the subscriber what has happened in the domain.

The value of crunch is that you can separate your own business domain from other services, and communicate through schemas and events rather than depending on each others libraries.

## Usage

See [examples](crates/crunch/examples/) for a more holistic listing of features

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let crunch = crunch::Builder::default().build()?;

    crunch.subscribe(|event| async move {
        println!("received event: {:?}", event);

        Ok(())
    })
    .await?;

    crunch
        .publish(SomeEvent {
            name: "some-name".into(),
        })
        .await?;

    Ok(())
}
```

This will publish `SomeEvent` as a domain event. The API is subject to change, as the codegen is being built.

## Workflow

Domain events works off of the principle that your interface is protobuf, as such your service will publish protobuf events, and subscribe to other services if needed.

To handle this workflow we introduce `crunch.toml` a file to manage these relationships.

Bootstrap the file using `crunch init`, see [crunch cli](crates/crunch-cli) for more info, it can automatically discover and add subscriptions, bump version, publish schemas etc.

```toml
[service]
service = "users-creation"
domain = "users"
codegen = ["rust"]

[[publish]]	
path = "crates/users-service/crunch"

[[subscription]]
service = "onboarding-signup"
domain = "onboarding"
version = "1.0.1"
output-path = "crates/users-service/crunch"
```

See [docs](docs/index.md) for more information (TBA)

## Tooling

When crunch is used in services it needs some supportive tooling, it isn't a requirement, but it helps ease development when using them.

- [x] [Cli](crates/crunch-cli) Used to generate code, add subscriptions, publish event schema, bump versions and more
  - [x] Codegen done (at least for an alpha)
  - [ ] Rest
- [x] [Codegen](crates/crunch-codegen) Can be used to automatically generate rust code depending on your crunch.toml file
  - [x] Main serialization and protobuf -> rust
  - [ ] Domain information

## Extensions

At its heart crunch is just a opinionated transport protocol, as such additional packages can be added for various needs:

- [ ] [Replay](crates/crunch-replay) Stores events in a replay store, so that consumers can choose to replay them whenever they want
- [ ] [Eventsource](crates/crunch-eventsource) Allows proper eventsourcing for the application, check the readme for more info
- [ ] [Cuddle](crates/crunch-cuddle) Will read overlapping values from .cuddle.yaml, as such we don't have to define service, domain multiple times, this also allows inheritance for certain fields. 

## Features

Crunch will is configurable to a variety of different transports, persistence layers and more. We recommend a few of them, which crunch automatically ships with (opt out via. `default-features = false` in Cargo.toml)

See the docs for each of them, to see how they can be enabled, and how to setup settings for them. If using `crunch-cuddle`, these settings can be inherited from a `cuddle-component`

We recommend wrapping and exposing the parts you need to the library, so that your services uses a consistent and opinionated layer on top of crunch.

### Transport

You will need a transport of some sort. Transport is what transfers messages between services, crunch is built to be configurable, and unopinionated, as such most messaging protocols should work fine. 

- [x] [NATS (recommended)](crates/crunch-transport-nats)
- [x] [Tokio channel (used for in-memory processing)](crates/crunch-transport-tokio-channel)

### Persistence

Crunch will need a persistence layer, like the other components these can be swapped in

- [ ] [PostgreSQL (recommended)](crates/crunch-postgres)
- [x] [In memory (used for in-memory processing)](crates/crunch-in-memory)
