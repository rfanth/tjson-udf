-- These are stored functions, they only work for this database.
-- requires (on just the currently connected database)
-- CREATE ROUTINE, or, alternately,
-- both CREATE FUNCTION and DROP FUNCTION
DROP FUNCTION IF EXISTS json_to_tjson_checked;
DROP FUNCTION IF EXISTS tjson_to_json_checked;

-- Validates options and raises a SQL error on any kind of bad input
-- containing the detailed error message from the underlying library.
-- This is MUCH slower than calling the udf's above directly.
-- For default options, pass NULL as opts, as this
-- is a stored function not a UDF and therefore sadly isn't variadic.
DELIMITER ;;
CREATE FUNCTION json_to_tjson_checked(s JSON, opts TEXT)
RETURNS TEXT
BEGIN
    DECLARE err TEXT;
    DECLARE msg TEXT;
    DECLARE out_tjson LONGTEXT;
    SET err = tjson_options_check(opts);
    IF err IS NOT NULL THEN
        SET msg = CONCAT_WS(' ','invalid tjson options:', err);
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = msg;
    END IF;
    SET out_tjson = json_to_tjson(s, opts);
    IF out_tjson IS NOT NULL OR s IS NULL THEN
        RETURN out_tjson;
    END IF;
    SET err = json_to_tjson_err(s, opts);
    SET msg = CONCAT_WS(' ', 'invalid json:', err);
    SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = msg;
END;;
DELIMITER ;

-- Validates options and raises a SQL error on any kind of bad input
-- containing the detailed error message from the underlying library.
-- This is MUCH slower than calling the udf's above directly.
DELIMITER ;;
CREATE FUNCTION tjson_to_json_checked(s JSON)
RETURNS TEXT
BEGIN
    DECLARE err TEXT;
    DECLARE msg TEXT;
    DECLARE out_json LONGTEXT;
    SET out_json = tjson_to_json(s);
    IF out_json IS NOT NULL OR s IS NULL THEN
        RETURN out_json;
    END IF;
    SET err = tjson_to_json_err(s);
    SET msg = CONCAT_WS(' ', 'invalid tjson:', err);
    SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = msg;
END;;
DELIMITER ;