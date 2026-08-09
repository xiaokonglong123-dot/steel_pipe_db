-- 018_drop_welded_pipes.sql
-- The welded_pipes master table (API 5L welded pipes) is REMOVED in the
-- SQLite + generic-ERP rewrite. All product data now lives in `items` (see
-- 002_create_items.sql).
--
-- The DROP is defensive: on a fresh database there is nothing to drop, but it
-- guarantees the legacy table cannot reappear if a stale development database
-- is reused.
DROP TABLE IF EXISTS welded_pipes;
