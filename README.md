# PI Home Info

Display of home information on Raspberry PI

## TIBBER

Get GraphQL schema

```bash
curl -s 'https://api.tibber.com/v1-beta/gql' -H 'Content-Type: application/json' -H 'Accept: application/json' -H "Authorization: Bearer $TIBBER"  --compressed --data-binary "{\"query\":\"`cat graphql/introspection.graphql | tr -d '\n\r' | tr -d '\n'`\"}" | jq . > graphql/tibber_schema.json
```
