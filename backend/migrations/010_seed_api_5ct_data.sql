-- 010_seed_api_5ct_data.sql
-- Seed data for API 5CT steel grade reference table.
-- Contains standard grades (H40, J55, K55, N80, L80, C90, T95, P110, Q125) with
-- their minimum yield/tensile strength (ksi) and chemical composition limits.
-- Uses INSERT OR IGNORE to be idempotent — safe to re-run.
-- Source: API Spec 5CT, 10th Edition.
INSERT OR IGNORE INTO api_5ct_grade_ref (grade, yield_strength_min, yield_strength_max, tensile_strength_min, carbon_content_max, manganese_content_max, phosphorus_content_max, sulfur_content_max) VALUES
('H40', 40.0, 80.0, 60.0, NULL, NULL, NULL, NULL),
('J55', 55.0, 80.0, 75.0, NULL, NULL, NULL, NULL),
('K55', 55.0, 80.0, 95.0, NULL, NULL, NULL, NULL),
('N80', 80.0, 110.0, 100.0, NULL, NULL, NULL, NULL),
('L80', 80.0, 95.0, 95.0, 0.50, 1.90, 0.030, 0.030),
('C90', 90.0, 105.0, 100.0, 0.50, 1.90, 0.030, 0.030),
('T95', 95.0, 110.0, 105.0, 0.50, 1.90, 0.030, 0.030),
('P110', 110.0, 140.0, 125.0, NULL, NULL, NULL, NULL),
('Q125', 125.0, 150.0, 135.0, NULL, NULL, NULL, NULL);
