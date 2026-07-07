//! Criterion benchmarks for Steel Pipe DB — JSON serialization, password hashing, JWT generation.
//!
//! Run with: `cargo bench --bench api_bench`

use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::Argon2;
use criterion::{criterion_group, criterion_main, Criterion};
use jsonwebtoken::{encode, Header};
use rand_core::OsRng;
use serde::Serialize;

// --- Test data ---

#[derive(Serialize)]
struct MockUser {
    id: i64,
    username: String,
    role: String,
    display_name: String,
}

#[derive(Serialize)]
struct MockApiResponse {
    success: bool,
    request_id: String,
    data: MockUser,
}

#[derive(Serialize)]
struct MockMeta {
    total: u64,
    page: u64,
    page_size: u64,
    total_pages: u64,
}

#[derive(Serialize)]
struct MockPaginatedItem {
    id: i64,
    pipe_number: String,
    grade: String,
    status: String,
}

#[derive(Serialize)]
struct MockPaginatedData {
    items: Vec<MockPaginatedItem>,
    total: u64,
    page: u64,
    page_size: u64,
    total_pages: u64,
}

#[derive(Serialize)]
struct MockPaginatedResponse {
    success: bool,
    request_id: String,
    meta: MockMeta,
    data: MockPaginatedData,
}

#[derive(Serialize)]
struct MockClaims {
    sub: i64,
    username: String,
    role: String,
    exp: usize,
    iat: usize,
}

fn bench_json_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_serialization");

    // Single object (ApiResponse<SeamlessPipe>)
    let single = MockApiResponse {
        success: true,
        request_id: "req_550e8400-e29b-41d4-a716-446655440000".into(),
        data: MockUser {
            id: 1,
            username: "admin".into(),
            role: "admin".into(),
            display_name: "系统管理员".into(),
        },
    };
    group.bench_function("single_object_1kb", |b| {
        b.iter(|| serde_json::to_string(&single).unwrap());
    });

    // Paginated list (20 items)
    let items: Vec<MockPaginatedItem> = (0..20)
        .map(|i| MockPaginatedItem {
            id: i,
            pipe_number: format!("SP-2025-{:04}", i),
            grade: "J55".into(),
            status: "in_stock".into(),
        })
        .collect();

    let paginated = MockPaginatedResponse {
        success: true,
        request_id: "req_550e8400-e29b-41d4-a716-446655440000".into(),
        meta: MockMeta {
            total: 100,
            page: 1,
            page_size: 20,
            total_pages: 5,
        },
        data: MockPaginatedData {
            items,
            total: 100,
            page: 1,
            page_size: 20,
            total_pages: 5,
        },
    };
    group.bench_function("paginated_20_items", |b| {
        b.iter(|| serde_json::to_string(&paginated).unwrap());
    });

    // Large paginated list (100 items)
    let large_items: Vec<MockPaginatedItem> = (0..100)
        .map(|i| MockPaginatedItem {
            id: i,
            pipe_number: format!("SP-2025-{:04}", i),
            grade: "J55".into(),
            status: "in_stock".into(),
        })
        .collect();

    let large_paginated = MockPaginatedResponse {
        success: true,
        request_id: "req_550e8400-e29b-41d4-a716-446655440000".into(),
        meta: MockMeta {
            total: 500,
            page: 1,
            page_size: 100,
            total_pages: 5,
        },
        data: MockPaginatedData {
            items: large_items,
            total: 500,
            page: 1,
            page_size: 100,
            total_pages: 5,
        },
    };
    group.bench_function("paginated_100_items", |b| {
        b.iter(|| serde_json::to_string(&large_paginated).unwrap());
    });

    group.finish();
}

fn bench_password_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("password_hashing");
    let password = "admin123";
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    // Hash password
    group.bench_function("argon2_hash", |b| {
        b.iter(|| {
            let salt = SaltString::generate(&mut OsRng);
            argon2.hash_password(password.as_bytes(), &salt).unwrap();
        });
    });

    // Verify password (pre-computed hash)
    let precomputed_hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();
    group.bench_function("argon2_verify", |b| {
        b.iter(|| {
            use argon2::password_hash::PasswordVerifier;
            argon2
                .verify_password(password.as_bytes(), &precomputed_hash)
                .unwrap();
        });
    });

    group.finish();
}

fn bench_jwt_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("jwt_generation");

    let claims = MockClaims {
        sub: 1,
        username: "admin".into(),
        role: "admin".into(),
        exp: 1735689600,
        iat: 1735682400,
    };
    let secret = "benchmark_test_secret_key_for_jwt_signing_32bytes!";
    let key = jsonwebtoken::EncodingKey::from_secret(secret.as_bytes());

    group.bench_function("hs256_sign", |b| {
        b.iter(|| {
            encode(&Header::default(), &claims, &key).unwrap();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_json_serialization,
    bench_password_hashing,
    bench_jwt_generation
);
criterion_main!(benches);
