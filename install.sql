-- Drop existing functions if present (safe to re-run)

-- These will be removed from your current database - they are wrappers with friendly types
DROP FUNCTION IF EXISTS json_to_tjson_with;
DROP FUNCTION IF EXISTS json_to_tjson;
DROP FUNCTION IF EXISTS tjson_to_json;
-- These are UDF's, they will work for all databases
DROP FUNCTION IF EXISTS tjson_options_check;
DROP FUNCTION IF EXISTS json_to_tjson_str;
DROP FUNCTION IF EXISTS tjson_to_json_str;


-- Install UDF shared library functions
CREATE FUNCTION tjson_to_json_str RETURNS STRING SONAME 'libtjson_udf.so';
CREATE FUNCTION json_to_tjson_str RETURNS STRING SONAME 'libtjson_udf.so';
CREATE FUNCTION tjson_options_check RETURNS STRING SONAME 'libtjson_udf.so';

-- Wrappers with friendly types
CREATE FUNCTION tjson_to_json(s TEXT)
RETURNS JSON
RETURN tjson_to_json_str(s);

CREATE FUNCTION json_to_tjson(s JSON)
RETURNS TEXT
RETURN json_to_tjson_str(s);

-- Validates options and raises a SQL error on bad input
DELIMITER ;;
CREATE FUNCTION json_to_tjson_with(s JSON, opts TEXT)
RETURNS TEXT
BEGIN
    DECLARE err TEXT;
    DECLARE msg TEXT;
    SET err = tjson_options_check(opts);
    IF err IS NOT NULL THEN
        SET msg = CONCAT('invalid tjson options: ', err);
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = msg;
    END IF;
    RETURN json_to_tjson_str(s, opts);
END;;
DELIMITER ;
