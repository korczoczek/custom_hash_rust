Derives custom sha256 hashes

```
Usage: custom_hash [OPTIONS] [MESSAGE] [KEY] [INDEX]

Arguments:
  [MESSAGE]  Starting content of the message [default: ""]
  [KEY]      Key to be found in the resulting hash [default: 0]
  [INDEX]    Starting index of the search in base62 [default: 0]

Options:
  -m, --mode <MODE>    Checking mode [default: start] [possible values: start, scatter, chunk]
  -a, --all            Find as many examples at the current depth
  -c, --count <COUNT>  Starting count [default: 1]
  -h, --help           Print help (see more with '--help')
  -V, --version        Print version
```
