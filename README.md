# tjson-udf

MariaDB UDF bindings for [tjson-rs](https://crates.io/crates/tjson-rs), exposing TJSON parse and render as native SQL functions.

[TJSON](https://textjson.com) is a hyper-readable, round trip safe and data preserving substitute for JSON. See the [tjson](https://github.com/rfanth/tjson) repository for full details.

## Requirements

- MariaDB 10.4+ (tested against 11.4 on Debian Trixie)
- Rust toolchain to build

## Build

```bash
cargo build --release
sudo cp target/release/libtjson_udf.so /usr/lib/mysql/plugin/
```

## Install

Run `install.sql` against your database:

```bash
mysql -u root < install.sql
```

This registers the UDF shared library and creates the following functions:

| Function | Arguments | Returns | Description |
|---|---|---|---|
| `tjson_to_json(s)` | TJSON text | JSON | Parse TJSON, return native MariaDB JSON |
| `json_to_tjson(s)` | JSON | TEXT | Render JSON as TJSON with default options |
| `json_to_tjson_with(s, opts)` | JSON, options TEXT | TEXT | Render JSON as TJSON with custom options |

## Usage

```sql
-- Parse TJSON to JSON
SELECT tjson_to_json('{ name: "Alice", age: 30 }');

-- Render JSON as TJSON
SELECT json_to_tjson('{"name": "Alice", "age": 30}');

-- Render with options
SELECT json_to_tjson_with('{"name": "Alice"}', '{"canonical": true}');
SELECT json_to_tjson_with('{"body": "long text..."}', '{"wrapWidth": 80, "multilineStrings": true}');
```

## Options

`json_to_tjson_with` accepts a JSON options object using the same camelCase keys as the [tjson npm package](https://www.npmjs.com/package/@rfanth/tjson). All fields are optional.

| Option | Type | Description |
|---|---|---|
| `canonical` | bool | Compact, diff-friendly output (overrides other options) |
| `wrapWidth` | number | Target line wrap width (0 = unlimited) |
| `forceMarkers` | bool | Always emit indent markers |
| `bareStrings` | string | Bare string style |
| `bareKeys` | string | Bare key style |
| `inlineObjects` | bool | Prefer inline object rendering |
| `inlineArrays` | bool | Prefer inline array rendering |
| `multilineStrings` | bool | Allow multiline string rendering |
| `multilineStyle` | string | Multiline string style |
| `tables` | bool | Enable pipe table rendering |
| `tableFold` | bool | Fold long table rows |

See the [tjson-rs documentation](https://docs.rs/tjson-rs) for the full options list.

## Error handling

Errors return SQL NULL. Detailed error messages are written to the MariaDB error log (`/var/log/mysql/error.log` or `journalctl -u mariadb`).

## Uninstall

```sql
DROP FUNCTION IF EXISTS json_to_tjson_with;
DROP FUNCTION IF EXISTS json_to_tjson;
DROP FUNCTION IF EXISTS tjson_to_json;
DROP FUNCTION IF EXISTS json_to_tjson_str;
DROP FUNCTION IF EXISTS tjson_to_json_str;
DROP FUNCTION IF EXISTS tjson_options_check;
```

## Resources

- Website and online demo: [textjson.com](https://textjson.com)
- Specification: [tjson-specification.md](https://github.com/rfanth/tjson-spec/blob/master/tjson-specification.md)
- tjson-rs: [github.com/rfanth/tjson](https://github.com/rfanth/tjson)

## License

BSD-3-Clause, same as tjson-rs.
