#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
plugin_dir="/usr/lib/mysql/plugin"

build_args=(build --release)
install_udfs=true
install_stored_functions=false
uninstall=false

usage() {
    cat <<'EOF'
Usage: ./install.sh [--error-log] [--plugin-dir DIR] [--stored|--all] [--uninstall] [-- MYSQL_COMMAND...]

Options:
  --error-log              Build with the Cargo feature "error-log" enabled.
  --plugin-dir DIR         Copy libtjson_udf.so into DIR instead of /usr/lib/mysql/plugin.
  --stored                 Install only install_stored_functions.sql. Pass a database with MYSQL_COMMAND, for example `-- mysql -u root -D mydb`.
  --all                    Install both UDFs and stored functions.
  --uninstall              Remove the installed SQL objects instead of installing them.
  -h, --help               Show this help.

Examples:
  ./install.sh
  ./install.sh --plugin-dir /custom/plugin/dir
  ./install.sh --stored -- mysql -u root -D mydb
  ./install.sh --all -- mysql -u root -D mydb
  ./install.sh --uninstall --stored -- mysql -u root -D mydb

MYSQL_COMMAND defaults to "mysql -u root".  If provided, it is used to run each SQL file,
with the file contents sent to its stdin.
It is normal for the installer to drop functions that do not exist.  It has to be careful because
overwriting a live plugin.so can cause the database to crash or otherwise misbehave.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --error-log)
            build_args+=(--features error-log)
            shift
            ;;
        --plugin-dir)
            if [[ $# -lt 2 ]]; then
                echo "--plugin-dir requires a directory argument" >&2
                usage >&2
                exit 1
            fi
            plugin_dir="$2"
            shift 2
            ;;
        --stored)
            install_udfs=false
            install_stored_functions=true
            shift
            ;;
        --all)
            install_udfs=true
            install_stored_functions=true
            shift
            ;;
        --uninstall)
            uninstall=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        --)
            shift
            break
            ;;
        *)
            break
            ;;
    esac
done

mysql_cmd=("$@")
if [[ ${#mysql_cmd[@]} -eq 0 ]]; then
    mysql_cmd=(mysql -u root)
fi

mysqlload() {
    local sql_file="$1"
    (
        cd "$repo_root"
        "${mysql_cmd[@]}" < "$sql_file"
    )
}

if [[ "$uninstall" == true ]]; then
    if [[ "$install_udfs" == true ]]; then
        echo "Uninstalling UDF SQL objects..."
        mysqlload uninstall_udf.sql
    fi

    if [[ "$install_stored_functions" == true ]]; then
        echo "Uninstalling stored functions..."
        mysqlload uninstall_stored_functions.sql
    fi
else
    echo "Building libtjson_udf.so..."
    (cd "$repo_root" && cargo "${build_args[@]}")

    if [[ "$install_udfs" == true ]]; then
        echo "Uninstalling UDF SQL objects before replacing the shared object..."
        mysqlload uninstall_udf.sql
    fi

    if [[ "$install_stored_functions" == true ]]; then
        echo "Uninstalling stored functions..."
        mysqlload uninstall_stored_functions.sql
    fi

    if [[ "$install_udfs" == true ]]; then
        echo "Copying libtjson_udf.so to $plugin_dir..."
        sudo cp "$repo_root/target/release/libtjson_udf.so" "$plugin_dir/"
    fi

    if [[ "$install_udfs" == true ]]; then
        echo "Installing UDF SQL objects...  (function does not exist messages are normal)"
        mysqlload install_udf.sql
    fi

    if [[ "$install_stored_functions" == true ]]; then
        echo "Installing stored functions...  (function does not exist messages are normal)"
        mysqlload install_stored_functions.sql
    fi
fi

echo "Done."
