# csvdelta

`csvdelta` is a fast command-line tool that compares two CSV files and reports row, cell, and column changes.

## Example

```console
$ cargo run -- examples/inventory_old.csv examples/inventory_new.csv --key sku
Added rows:
  SKU-1016: {"category": "Office", "name": "Label Printer", "price": "129.00", "sku": "SKU-1016", "stock": "6"}
Removed rows:
  SKU-1014: {"category": "Office", "name": "Notebook A5", "price": "3.40", "sku": "SKU-1014", "stock": "48"}
Changed cells:
  SKU-1001 / stock: "42" -> "36"
  SKU-1003 / stock: "65" -> "72"
  SKU-1004 / price: "19.95" -> "21.50"
  SKU-1006 / stock: "75" -> "61"
  SKU-1007 / price: "24.50" -> "22.95"
  SKU-1011 / stock: "200" -> "180"
  SKU-1015 / price: "29.99" -> "27.50"
  SKU-1015 / stock: "9" -> "16"
Columns added: none
Columns removed: none
1 added, 1 removed, 8 cells changed in 7 rows
```

The command exits with status `1` when differences are found; the example can be run with `cargo run -- ...` without masking that status.

## Installation

Install from a local checkout:

```sh
cargo install --path .
```

Or build the Docker image and invoke the command through it:

```sh
docker build -t csvdelta .
docker run --rm -v "$PWD:/data" csvdelta /data/examples/inventory_old.csv /data/examples/inventory_new.csv --key sku
```

This is a command-line tool, so a live deployment does not apply. Docker is the portable distribution option.

## Usage

```text
csvdelta <OLD.csv> <NEW.csv> [--key <column>] [--format <table|json|markdown>]
```

- `OLD.csv`, `NEW.csv`: CSV files with header rows. Quoted commas and embedded newlines are supported.
- `--key <column>`: column used to match rows. Defaults to the first column of the old CSV.
- `--format <table|json|markdown>`: output format; defaults to `table`.

Columns are matched by header name regardless of their order. Added and removed columns are reported; cell changes are compared only in columns present in both files. Duplicate keys and malformed CSV input are errors.

### Exit codes

| Code | Meaning |
|---|---|
| `0` | No differences |
| `1` | Differences found |
| `2` | Error (for example, missing input, empty file, missing key column, or duplicate key) |

## Tests

```sh
cargo test
# or
make test
```

To test the container build:

```sh
make docker-test
```

## Project layout

```text
src/main.rs       CLI arguments, CSV input, and exit codes
src/lib.rs        library exports
src/diff.rs       comparison logic and unit tests
src/output.rs     table, JSON, and Markdown renderers
tests/            integration tests and CSV fixtures
examples/         inventory demonstration data
```

## License

MIT. See [LICENSE](LICENSE).
