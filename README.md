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

Using tikv-jemalloc:
```bash
echo "GET http://localhost:3000/health" | vegeta attack -rate=0 -max-workers=300 -duration=10s | vegeta report
Requests      [total, rate, throughput]         771694, 77169.25, 77162.04
Duration      [total, attack, wait]             10.001s, 10s, 933.94µs
Latencies     [min, mean, 50, 90, 95, 99, max]  48.702µs, 2.856ms, 2.633ms, 5.127ms, 6.027ms, 7.87ms, 15.843ms
Bytes In      [total, mean]                     0, 0.00
Bytes Out     [total, mean]                     0, 0.00
Success       [ratio]                           100.00%
Status Codes  [code:count]                      200:771694
Error Set:
```

Using default alloc:
```bash
Requests      [total, rate, throughput]         767826, 76781.93, 76771.54
Duration      [total, attack, wait]             10.001s, 10s, 1.354ms
Latencies     [min, mean, 50, 90, 95, 99, max]  41.444µs, 2.952ms, 2.701ms, 5.18ms, 6.043ms, 7.763ms, 17.298ms
Bytes In      [total, mean]                     0, 0.00
Bytes Out     [total, mean]                     0, 0.00
Success       [ratio]                           100.00%
Status Codes  [code:count]                      200:767826
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
