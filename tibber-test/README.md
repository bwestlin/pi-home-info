# Tibber test

Run with:

```bash
cargo run
```

`TIBBER` env var with token needs to be set

Generate schema with:

```bash
cynic introspect --server-version 2021 https://api.tibber.com/v1-beta/gql -o tibber.graphql
```
