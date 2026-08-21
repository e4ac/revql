[![CICD](https://github.com/e4ac/revql/actions/workflows/cicd.yml/badge.svg)](https://github.com/e4ac/revql/actions/workflows/cicd.yml)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

# RevQL
GraphQL object reverse lookup tool. The goal of this tool is to:
- Search for all queries/mutations to specific objects
- Search for all queries/mutations to any object containing a specific field

## Installing
```
cargo install revql --git https://github.com/e4ac/revql/
```

## Usage
```
Usage: revql [OPTIONS] <FILE> <SEARCH>

Arguments:
  <FILE>    Specifies the path to the JSON file containing the introspection result
  <SEARCH>  Specifies the search term

Options:
  -o, --output-file <OUTPUT_FILE>  Specifies the output file path. Print to console if not specified
      --no-color                   Specifies whether to disable color output
      --search-mode <SEARCH_MODE>  Specifies the search mode [default: type] [possible values: type, description, field]
      --contains                   Specifies whether to perform a "contains" match instead of an exact match
      --ignore-case                Specifies whether to ignore cases when searching
  -u, --url                        Specifies that <FILE> is a URL instead of a schema file
  -H, --header <HEADERS>           Specifies the header to include in the request
      --query-file <QUERY_FILE>    Specifies the path to a custom `.graphql` file for the introspection query
      --query <QUERY>              Specifies a custom introspection query
  -X, --request <METHOD>           Specifies the HTTP method to use [default: POST] [possible values: GET, POST]
  -h, --help                       Print help
```

## Examples
Search for all queries/mutations to a type named `User`:
```sh
revql schema.json User
```

Search for all queries/mutations to any type with `User` in it:
```sh
revql --contains schema.json User
```

Search for all queries/mutations to any type with a field named `username`:
```sh
revql schema.json username --search-mode field
```

Search for all queries/mutations to any type/field with a description `some desc`:
```sh
revql schema.json 'some desc' --search-mode description --contains
```

Search for all queries/mutations to a type named `User` using a URL:
```sh
revql --url https://graphqlzero.almansi.me/api User
```

Search for all queries/mutations to a type named `User` using a URL with GET:
```sh
revql --url https://graphqlzero.almansi.me/api User -X GET -H 'Content-Type: application/json' -H 'apollo-require-preflight: true'
```
