-- These are stored functions, they only work for this database.
-- requires (on just the currently connected database)
-- CREATE ROUTINE, ALTER ROUTINE, or DROP FUNCTION.
-- The function owner can also DROP them.
DROP FUNCTION IF EXISTS json_to_tjson_checked;
DROP FUNCTION IF EXISTS tjson_to_json_checked;