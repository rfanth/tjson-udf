-- These are all UDF's, they work for all databases.
-- This requires GRANT INSERT and GRANT DELETE on mysql.func to CREATE/DROP these UDF's
DROP FUNCTION IF EXISTS tjson_to_json;
DROP FUNCTION IF EXISTS tjson_to_json_err;
DROP FUNCTION IF EXISTS json_to_tjson;
DROP FUNCTION IF EXISTS json_to_tjson_err;
DROP FUNCTION IF EXISTS tjson_options_check;

CREATE FUNCTION tjson_to_json RETURNS STRING SONAME 'libtjson_udf.so';
CREATE FUNCTION tjson_to_json_err RETURNS STRING SONAME 'libtjson_udf.so';
CREATE FUNCTION json_to_tjson RETURNS STRING SONAME 'libtjson_udf.so';
CREATE FUNCTION json_to_tjson_err RETURNS STRING SONAME 'libtjson_udf.so';
CREATE FUNCTION tjson_options_check RETURNS STRING SONAME 'libtjson_udf.so';
