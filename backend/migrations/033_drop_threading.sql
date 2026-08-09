-- 033_drop_threading.sql
-- The pipe threading module (machining records + thread geometry cache) is
-- REMOVED in the SQLite + generic-ERP rewrite — thread math is pipe-specific.
-- The DROPs are defensive: on a fresh database there is nothing to drop, but
-- they guarantee these legacy tables cannot reappear if a stale development
-- database is reused.
DROP TABLE IF EXISTS threading_records;
DROP TABLE IF EXISTS thread_geometry_cache;
