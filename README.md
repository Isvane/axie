# axie

An async Rust backend service built to go beyond simple tutorial projects and handle real-world API stuff properly.

---

## Quick Start

```bash
# Environment Setup
cp .env.example .env

# Launch Service & Postgres Container
docker compose up --build

# Run Test (in-memory)
cargo test
```

---

## Architecture

* **Axum & Tokio:** Built on Axum web framework powered by Tokio's async runtime.
* **Casbin Authorization Engine:** Uses `casbin` (RBAC model) wrapped in an `Arc<RwLock<Enforcer>>` inside app state. Requests to protected routes pass through custom Axum middleware (`casbin_enforce`) that checks the request path, HTTP method, and user role against policy rules.
* **Non-blocking Auth & Lazy Keys:** Argon2 password hashing runs off the async thread pool using `tokio::task::spawn_blocking` to prevent event loop starvation. JWT encoding/decoding keys are lazily initialized at startup using `std::sync::LazyLock`.
* **Database & Automatic Migrations:** Powered by `Postgres` and `toasty` ORM with embedded migrations (`toasty::embed_migrations!`) applied automatically on startup.
* **Traffic & Request Safeguards:**
  * Rate-limiting via `tower-governor` using `SmartIpKeyExtractor`.
  * Request timeout protection (`TimeoutLayer`) capping requests at 10 seconds.
  * Structured tracing via `tower-http` (`TraceLayer`) and `tracing-subscriber`.
* **Jemalloc Allocator:** Overrides the default memory allocator with `tikv-jemallocator` on non-MSVC targets for enhanced memory performance under high concurrent load.

---

## Benchmarks

Layer were turned off and logging were also set to off on the dockerfile when running this benchmark.

Using tikv-jemalloc:
```bash
echo "GET http://localhost:3000/health" | vegeta attack -rate=0 -max-workers=300 -duration=10s | vegeta report
Requests      [total, rate, throughput]         759572, 75951.77, 75951.07
Duration      [total, attack, wait]             10.001s, 10.001s, 92.092µs
Latencies     [min, mean, 50, 90, 95, 99, max]  47.123µs, 2.968ms, 2.689ms, 5.433ms, 6.359ms, 8.315ms, 17.393ms
Bytes In      [total, mean]                     0, 0.00
Bytes Out     [total, mean]                     0, 0.00
Success       [ratio]                           100.00%
Status Codes  [code:count]                      200:759572
Error Set:
```

Using default alloc:
```bash
echo "GET http://localhost:3000/health" | vegeta attack -rate=0 -max-workers=300 -duration=10s | vegeta report
Requests      [total, rate, throughput]         727057, 72702.34, 72694.08
Duration      [total, attack, wait]             10.002s, 10s, 1.136ms
Latencies     [min, mean, 50, 90, 95, 99, max]  43.67µs, 3.178ms, 2.924ms, 5.656ms, 6.579ms, 8.455ms, 29.323ms
Bytes In      [total, mean]                     0, 0.00
Bytes Out     [total, mean]                     0, 0.00
Success       [ratio]                           100.00%
Status Codes  [code:count]                      200:727057
Error Set:
```

---

## API Routes

| Method | Endpoint | Description | Extractors / Middleware |
| --- | --- | --- | --- |
| **GET** | `/health` | Liveness / Health check endpoint | None |
| **GET** | `/` | Root Index | None |
| **GET** | `/pages` | Query-driven list pagination | `Query<Pagination>` |
| **POST** | `/login` | Authenticate and issue JWT | `Json<AuthPayload>` |
| **GET** | `/users/` | User section about | None |
| **POST** | `/users/create` | Validate and insert new user | `Json<CreateUser>` |
| **PATCH** | `/users/update/{id}` | Update user profile | **JWT (`Claims`)** + `Path<u64>` + `Json<UpdateUser>` |
| **DELETE** | `/users/delete/{id}` | Remove a user by ID | **JWT (`Claims`)** + `Path<u64>` |
| **GET** | `/users/greet/{name}` | Dynamic path injection | `Path<String>` |
| **GET** | `/admin/list` | Fetch all users | **Casbin Enforcer** + **JWT (`Claims`)** + `Query<Pagination>` |
| **PATCH** | `/admin/{id}/role` | Modify user role level | **Casbin Enforcer** + **JWT (`Claims`)** + `Path<u64>` + `Json<ChangeRolePayload>` |
| **POST** | `/owner/transfer-ownership` | Transfer company ownership | **Casbin Enforcer** + **JWT (`Claims`)** + `Json<TransferOwnershipPayload>` |
| **POST** | `/owner/rename-company` | Update company name for all members | **Casbin Enforcer** + **JWT (`Claims`)** + `Json<UpdateCompanyPayload>` |
| **ANY** | `/assets/*` | Static asset serving | `ServeDir` ("public") |
| **ANY** | `/*` | SPA Fallback Route | `ServeDir` / `ServeFile` ("public/index.html") |
