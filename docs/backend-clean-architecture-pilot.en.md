# Backend Clean Architecture Pilot

## Goal

This pilot keeps backend behavior unchanged while introducing explicit architectural boundaries for the `supplier` vertical slice.

## New Folder Structure

```text
backend/src/
├── domain/
│   └── supplier.rs
├── application/
│   └── suppliers/
│       └── service.rs
├── interface/
│   └── http/
│       └── suppliers/
│           ├── dto.rs
│           └── handler.rs
└── infrastructure/
    └── persistence/
        └── suppliers/
            └── repository.rs
```

## Boundary Mapping

- `domain/` owns core business data shapes such as `Supplier`.
- `application/` owns use-case orchestration through `SupplierService`.
- `interface/http/` owns HTTP DTOs and handler entrypoints.
- `infrastructure/persistence/` owns SQLx repository code.

## Compatibility Strategy

The legacy files remain in place as compatibility facades:

- `handlers/supplier_handler.rs`
- `services/supplier_service.rs`
- `repositories/supplier_repo.rs`
- `dto/supplier_dto.rs`
- `models/supplier.rs`

Each old path re-exports the new implementation so the crate can migrate incrementally instead of forcing a repo-wide rewrite.

## Why This Is Safe

- Route function names stay unchanged.
- Handler/service/repository macros stay unchanged.
- Axum `Extension<SqlitePool>` wiring stays unchanged.
- Existing imports continue to compile through facades.

## Next Migration Steps

1. Move `customer` into the same boundaries.
2. Group remaining CRUD slices by bounded context.
3. Retire compatibility facades only after all imports are migrated.
