# axie

An async Rust backend service built to go beyond simple tutorial projects and handle real-world API stuff "properly".

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
* **Casbin Authorization Engine:** Uses `casbin` (RBAC model) wrapped in an `Arc<Enforcer>` inside app state. Requests to protected routes pass through custom Axum middleware (`casbin_enforce`) that checks the request path, HTTP method, and user role against policy rules.
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
Hardware: **i5-10300H**.

Using tikv-jemalloc:
```bash
# PATCH http://localhost:3000/users/update/1
Requests      [total, rate, throughput]         7500, 250.03, 250.02
Duration      [total, attack, wait]             29.997s, 29.996s, 1.382ms
Latencies     [min, mean, 50, 90, 95, 99, max]  1.196ms, 1.484ms, 1.395ms, 1.454ms, 1.474ms, 2.534ms, 37.881ms
Bytes In      [total, mean]                     112500, 15.00
Bytes Out     [total, mean]                     435000, 58.00
Success       [ratio]                           100.00%
Status Codes  [code:count]                      200:7500
Error Set:
```

Using default alloc:
```bash
# PATCH http://localhost:3000/users/update/1
Requests      [total, rate, throughput]         7500, 250.03, 249.13
Duration      [total, attack, wait]             30.105s, 29.996s, 109.325ms
Latencies     [min, mean, 50, 90, 95, 99, max]  1.255ms, 8.588ms, 1.461ms, 2.037ms, 63.537ms, 152.196ms, 345.072ms
Bytes In      [total, mean]                     112500, 15.00
Bytes Out     [total, mean]                     435000, 58.00
Success       [ratio]                           100.00%
Status Codes  [code:count]                      200:7500
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
| **GET** | `/user/` | User section about | None |
| **POST** | `/user/create` | Validate and insert new user | `Json<CreateUser>` |
| **PATCH** | `/user/{id}` | Update user profile | **JWT (`Claims`)** + `Path<u64>` + `Json<UpdateUser>` |
| **DELETE** | `/user/{id}` | Remove a user by ID | **JWT (`Claims`)** + `Path<u64>` |
| **GET** | `/user/greet/{name}` | Dynamic path injection | `Path<String>` |
| **GET** | `/admin/list` | Fetch all users | **Casbin Enforcer** + **JWT (`Claims`)** + `Query<Pagination>` |
| **PATCH** | `/admin/{id}/role` | Modify user role level | **Casbin Enforcer** + **JWT (`Claims`)** + `Path<u64>` + `Json<ChangeRolePayload>` |
| **POST** | `/owner/transfer-ownership` | Transfer company ownership | **Casbin Enforcer** + **JWT (`Claims`)** + `Json<TransferOwnershipPayload>` |
| **POST** | `/owner/rename-company` | Update company name for all members | **Casbin Enforcer** + **JWT (`Claims`)** + `Json<UpdateCompanyPayload>` |
| **ANY** | `/assets/*` | Static asset serving | `ServeDir` ("public") |
| **ANY** | `/*` | SPA Fallback Route | `ServeDir` / `ServeFile` ("public/index.html") |
