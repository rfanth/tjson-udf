# tjson-udf

MariaDB UDF bindings for [tjson-rs](https://crates.io/crates/tjson-rs), exposing TJSON parse and render as native SQL functions.

[TJSON](https://textjson.com) is a hyper-readable, round trip safe and data preserving substitute for JSON. See the [tjson](https://github.com/rfanth/tjson) repository for full details.

## Requirements

- MariaDB 10.4+ (tested against 11.4 on Debian Trixie)
- Rust toolchain to build

## Build and install

If you have not downloaded the source yet, copy and paste these commands:

```bash
git clone https://github.com/rfanth/tjson-udf
cd tjson-udf
```

The easiest way to install is to use the included installer script.

**Install UDFs only**:

```bash
./install.sh
```

**Install UDFs and the checked stored functions**:

```bash
./install.sh --all -- mysql -u root -D mydb
```

**Build with MariaDB error-log logging enabled**:

```bash
./install.sh --error-log
```

Replace `mydb` with your database name.

What `install.sh` does:

- Builds `target/release/libtjson_udf.so`
- Removes the existing SQL objects first for safety
- Copies the shared object into MariaDB's plugin directory
- Reinstalls the SQL objects you selected

Useful installer options:

- `--plugin-dir DIR` uses a plugin directory other than `/usr/lib/mysql/plugin`
- `--stored` installs only `install_stored_functions.sql`
- `--all` installs both the UDFs and the stored functions
- `--uninstall` removes the selected SQL objects instead of installing them

Examples:

```bash
./install.sh --plugin-dir /custom/plugin/dir
./install.sh --stored --uninstall -- mysql -u root -D mydb
```

If you prefer to do the steps manually, `install.sh` is just automating this process:

- `cargo build --release` or `cargo build --release --features error-log`
- copy `target/release/libtjson_udf.so` into MariaDB's plugin directory
- run `install_udf.sql`
- optionally run `install_stored_functions.sql` in a specific database

### Functions

| Function | Arguments | Returns | Description |
|---|---|---|---|
| `tjson_to_json(s)` | TJSON text | STRING | Parse TJSON, return JSON string. NULL on error. |
| `tjson_to_json_err(s)` | TJSON text | STRING | NULL on success, error message on failure. |
| `json_to_tjson(s[, opts])` | JSON, options JSON | STRING | Render JSON as TJSON. NULL on error. |
| `json_to_tjson_err(s[, opts])` | JSON, options JSON | STRING | NULL on success, error message on failure. |
| `tjson_options_check(opts)` | options JSON | STRING | NULL if options are valid or NULL, error message if invalid. |
| `json_to_tjson_checked(s, opts)` | JSON, options JSON | TEXT | Render JSON as TJSON. Raises a SQL error on any failure. Pass NULL for default options. |
| `tjson_to_json_checked(s)` | TJSON text | TEXT | Parse TJSON. Raises a SQL error on failure. |

The UDFs (`tjson_to_json`, `json_to_tjson`, and their `_err` variants, and `tjson_options_check`) are fast. The `_checked` stored functions call the UDFs internally and are significantly slower — use them for setup/config paths, not hot row processing.  All functions including _err variants and tjson_options_check propagate a NULL first argument.

## Usage

```sql
-- Parse TJSON to JSON
SELECT tjson_to_json('  name: Alice\n  age: 30');

-- Render JSON as TJSON
SELECT json_to_tjson('{"name": "Alice", "age": 30}');

-- Render with options
SELECT json_to_tjson('{"name": "Alice"}', '{"canonical": true}');

-- Find bad rows and see why
SELECT col, tjson_to_json_err(col) FROM t WHERE tjson_to_json(col) IS NULL;

-- Raise a SQL error on bad input (e.g. in a trigger or stored procedure)
SELECT json_to_tjson_checked('{"name": "Alice"}', '{"canonical": true}');
SELECT tjson_to_json_checked('  name: Alice');
```

## Options

`json_to_tjson` and `json_to_tjson_checked` accept a JSON options object using the same camelCase keys as the [tjson npm package](https://www.npmjs.com/package/@rfanth/tjson). All fields are optional.

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

See the [tjson-rs documentation](https://docs.rs/tjson-rs) for the full options list.

## Error handling

`tjson_to_json` and `json_to_tjson` return NULL on error. To get the error message, call the corresponding `_err` variant on the same input.

If you build with `cargo build --release --features error-log`, the UDFs also write errors to the MariaDB error log (`/var/log/mysql/error.log` or `journalctl -u mariadb`).

The `_checked` stored functions raise a SQL error (`SQLSTATE 45000`) with the error message, suitable for use in triggers or stored procedures where silent NULL propagation is undesirable.

## Uninstall

The easiest way to uninstall is with the installer script:

```bash
./install.sh --uninstall
./install.sh --uninstall --all -- mysql -u root -D mydb
```

Manual uninstall is also available.

**UDFs** — require `DELETE ON mysql.func`:

```bash
mysql -u root < uninstall_udf.sql
```

**Stored functions** — require `DROP ROUTINE` (or `ALTER ROUTINE`, or ownership) on the target database:

```bash
mysql -u root -D mydb < uninstall_stored_functions.sql
```

## Resources

- Website and online demo: [textjson.com](https://textjson.com)
- Specification: [tjson-specification.md](https://github.com/rfanth/tjson-spec/blob/master/tjson-specification.md)
- tjson-rs: [github.com/rfanth/tjson](https://github.com/rfanth/tjson)

## License

BSD-3-Clause, same as tjson-rs.
