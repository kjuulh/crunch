# Crunch - Domain event interface

Crunch allows services to distribute their own events, as domain events. Domain events are a more strict representation on the api between services in different domains, they are meant to tell the subscriber what has happened in the domain.

The value of crunch is that you can separate your own business domain from other services, and communicate through schemas and events rather than depending on each others libraries.

## Usage

See [examples](crates/crunch/examples/) for a more holistic listing of features

```rust
impl Event for SomeEvent {
    fn event_info(&self) -> EventInfo {
        EventInfo {
            domain: "some-domain",
            entity_type: "some-entity",
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let in_memory = Persistence::in_memory();
    OutboxHandler::new(in_memory.clone()).spawn();
    let publisher = Publisher::new(in_memory);

    publisher
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

```toml
[service]
codegen = ["rust"]

[[publish]]	
service = "users-creation"
domain = "users"
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

- [ ] [Cli](crates/crunch-cli)
- [ ] [Codegen](crates/crunch-codegen) Can be used to automatically generate rust code depending on your crunch.toml file

## Extensions

At its heart crunch is just a opinionated transport protocol, as such additional packages can be added for various needs:

- [ ] [Replay](crates/crunch-replay) Stores events in a replay store, so that consumers can choose to replay them whenever they want
- [ ] [Eventsource](crates/crunch-eventsource) Allows proper eventsourcing for the application, check the readme for more info
- [ ] [Cuddle](crates/crunch-cuddle) Will read overlapping values from .cuddle.yaml, as such we don't have to define service, domain multiple times, this also allows inheritance for certain fields. 

