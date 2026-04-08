-- Drop existing functions if present (safe to re-run)

-- These will be removed from your current database - they are wrappers with friendly types
DROP FUNCTION IF EXISTS json_to_tjson_with;
DROP FUNCTION IF EXISTS json_to_tjson;
DROP FUNCTION IF EXISTS tjson_to_json;

-- These are UDF's, they will work for all databases
DROP FUNCTION IF EXISTS tjson_options_check;
DROP FUNCTION IF EXISTS json_to_tjson_str;
DROP FUNCTION IF EXISTS tjson_to_json_str;
