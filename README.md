[![Build](https://github.com/sparkmeter/nquery/workflows/CI/badge.svg)](https://github.com/sparkmeter/nquery/actions?query=workflow%3ACI)
[![Latest version](https://img.shields.io/crates/v/nquery.svg?style=flat)](https://crates.io/crates/nquery)

# nquery
Query and explore jobs on your Nomad clusters from the comfort of the command line.

The output is raw JSON, to facilitate integration with tooling such
as [`jq`](https://stedolan.github.io/jq/).

## Usage

```bash
# Get all jobs with IDs starting with 'redis' as pretty-printed JSON
$ nquery --pretty redis
[
  {
    "ID": "redis",
    "Name": "redis",
    "Namespace": "default",
    "ParameterizedJob": null,
    # ...
    "TaskGroups": [
      {
        # ...
        "Tasks": [
           {
              # ...
              "Meta": null,
              "Name": "redis",
           }
        ]
      }
    ]
  }
]


# Get the ID and task data-source fields of all parameterized jobs starting with etl
$ nquery --pretty --parameterized -f Meta.data-source etl
[
  {
    "ID": "etl-cluster-1"
    "Meta.data-source": "db-cluster-1",
  },
  {
    "ID": "etl-cluster-2"
    "Meta.data-source": "db-cluster-2",
  }
]

# Count the number of ETL tasks
$ nquery --parameterized -f Meta.data-source etl | jq '. | length'
```

## Installation

[Download the latest binary for your platform from the releases page](https://github.com/sparkmeter/nquery/releases).

or, with Cargo:

```bash
cargo install nquery
```


## Debugging

To get helpful debugging information, run nquery with the `NQUERY_LOG=nquery`
environment variable set.
