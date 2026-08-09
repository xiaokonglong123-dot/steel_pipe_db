-- 007_drop_quality_pipe_tables.sql
-- The steel-pipe quality module is REMOVED in the SQLite + generic-ERP
-- rewrite:
--   - quality_certs        (pipe-type quality certificates)
--   - api_5ct_grade_ref    (API 5CT steel grade reference data)
--   - pipe_attachments     (file attachments bound to pipes)
-- Manufacturing quality inspection (mfg_inspections / mfg_ncrs in
-- 032_create_manufacturing.sql) is KEPT.
--
-- The DROPs are defensive: on a fresh database there is nothing to drop, but
-- they guarantee these legacy tables cannot reappear if a stale development
-- database is reused.
DROP TABLE IF EXISTS quality_certs;
DROP TABLE IF EXISTS api_5ct_grade_ref;
DROP TABLE IF EXISTS pipe_attachments;
