-- 020_set_pipe_status_default_new.sql
-- Change the status column DEFAULT from 'in_stock' to 'new' for all three
-- pipe tables so that any INSERT without an explicit status defaults to the
-- status machine's initial state instead of immediately being counted as stock.
--
-- The application-layer fix (build_create_query now always writes 'new') is
-- the load-bearing change; this migration only aligns the DB default so that
-- bare INSERTs (e.g. from ad-hoc scripts or future tooling) also get 'new'.
--
-- Existing rows keep their current status — mass-updating in_stock rows would
-- corrupt real inventory data.
--
-- SQLite 3.28+ supports ALTER COLUMN SET DEFAULT directly; no need for the
-- copy-drop-rename pattern used in 019 (which was needed for CHECK changes).

ALTER TABLE seamless_pipes ALTER COLUMN status SET DEFAULT 'new';
ALTER TABLE screen_pipes   ALTER COLUMN status SET DEFAULT 'new';
ALTER TABLE welded_pipes   ALTER COLUMN status SET DEFAULT 'new';
