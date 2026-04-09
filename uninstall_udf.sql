-- These are all UDF's, they work for all databases.
-- This requires GRANT DELETE on mysql.func to DROP these UDF's
DROP FUNCTION IF EXISTS tjson_to_json;
DROP FUNCTION IF EXISTS tjson_to_json_err;
DROP FUNCTION IF EXISTS json_to_tjson;
DROP FUNCTION IF EXISTS json_to_tjson_err;
DROP FUNCTION IF EXISTS tjson_options_check;
