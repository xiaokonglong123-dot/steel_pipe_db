-- 003_drop_screen_pipes.sql
-- The screen_pipes master table (API 5CT screen pipes) is REMOVED in the
-- SQLite + generic-ERP rewrite. All product data now lives in `items` (see
-- 002_create_items.sql).
--
-- The DROP below is defensive: on a fresh database there is nothing to drop,
-- but it guarantees the legacy table cannot reappear if a stale development
-- database is reused.
DROP TABLE IF EXISTS screen_pipes;
