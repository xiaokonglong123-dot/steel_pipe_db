Test Data Package for Steel Pipe DB

Overview
- This directory contains CSV data sets used for testing the web API (import/export) and for UI testing.
- Files are designed for deterministic results and easy repeatability.

File layout
- pipes_test.csv        -> Normal data for import/export tests
- pipes_error.csv       -> Invalid data to test validation and error handling
- pipes_export.csv      -> Generated during tests (example of an exported file)
- README.md              -> This document

CSV format and encoding
- Delimiter: comma (,)
- Header row: yes (column names exactly as defined below)
- Encoding: UTF-8
- Newline: CRLF or LF (both supported by most tools)

Column definitions
- diameter: REAL, unit millimeters (mm); must be > 0
- thickness: REAL, unit millimeters (mm); must be > 0
- length: REAL, unit meters (m); must be > 0
- material: TEXT; allowed values: Carbon Steel, Stainless Steel, Alloy Steel
- quantity: INTEGER; must be >= 0

Data validation rules
- All numeric fields must be parseable as numbers
- Non-numeric values in numeric columns should be rejected by the API (400) or ignored by import routines with error messages
- material must be one of the allowed values; otherwise error during import
- quantity must be an integer; non-integer values cause error during import

Example data (pipes_test.csv)
```csv
diameter,thickness,length,material,quantity
12.0,1.2,6.0,Carbon Steel,10
16.5,1.5,8.0,Stainless Steel,5
21.3,2.0,12.0,Alloy Steel,20
10.0,0.8,3.5,Carbon Steel,15
14.0,1.0,4.5,Stainless Steel,7
25.4,2.5,9.0,Alloy Steel,12
8.0,0.6,2.0,Carbon Steel,30
18.0,1.4,5.5,Stainless Steel,9
```

Testing guidance
- Import: use /pipes/import with pipes_test.csv
- Export: use /pipes/export?format=csv to generate a sample export
- Error handling: import pipes_error.csv to verify proper error reporting
- Exported file name is not fixed; you may name it pipes_export.csv during your test run

Versioning and maintenance
- This test data package is versioned alongside the API changes
- When API fields or validation rules change, update the test data accordingly

Notes
- Do not include real customer data in test data
- The sample values are chosen to cover common ranges and typical material types
