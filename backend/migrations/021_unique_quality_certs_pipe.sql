-- 021_unique_quality_certs_pipe.sql
-- Enforce one active quality certificate per pipe: de-duplicate any existing
-- rows first, then create a partial unique index that ignores soft-deleted certs.
--
-- Before this migration, `create_cert` could insert multiple certificates for
-- the same (pipe_type, pipe_id) — including conflicting results (e.g. one pass,
-- one fail) — silently corrupting quality records.
--
-- The index alone would fail on existing duplicates, so the deduplication
-- DELETE runs first. It keeps the lowest `id` row per (pipe_type, pipe_id)
-- among non-deleted certs and removes the rest. Soft-deleted rows are
-- untouched — they're already excluded from the partial index.

-- De-duplicate: keep only the row with the smallest `id` per (pipe_type, pipe_id)
-- for all non-soft-deleted certificates.
DELETE FROM quality_certs
WHERE deleted_at IS NULL
  AND id NOT IN (
    SELECT MIN(id) FROM quality_certs
    WHERE deleted_at IS NULL
    GROUP BY pipe_type, pipe_id
  );

-- Create a partial unique index that only enforces uniqueness on active certs.
-- Soft-deleted rows (deleted_at IS NOT NULL) are excluded so that historical
-- certs don't block re-certification of the same pipe.
CREATE UNIQUE INDEX IF NOT EXISTS uni_quality_certs_pipe
  ON quality_certs (pipe_type, pipe_id)
  WHERE deleted_at IS NULL;
