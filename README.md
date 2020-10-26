# nquery
Query and explore jobs on your Nomad clusters.

The output is raw JSON, to facilitate integration with command line tooling, such
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
```

## Installation

```bash
cargo install nquery
```

