# Dororo

An asynchronous backend sandbox built to master production-grade web services in Rust.

---

## Core Learning Takeaways

* **Thread-Safe State & Static Keys:** Sharing DB pools via `Arc<AppState>` and lazy initializing crypto keys with `std::sync::LazyLock`.
* **Type-Safe Extraction & RBAC:** Extracting Bearer tokens via `TypedHeader` into `Claims` paired with role-matching middleware (`require_role`).
* **Socketless Integration Testing:** Testing the HTTP pipeline in-memory via `tower::Service` (`oneshot`/`call`) using an ephemeral `sqlite::memory:` database.
* **Defensive Traffic Control:** Layering Tower middleware for global rate-limiting (`GovernorLayer`), timeouts (`TimeoutLayer`), and concurrency limits.
* **Declarative Payload Validation:** Binding the `validator` crate to deserialization pipelines to sanitize input data before hitting domain logic.

---

## API Endpoints

### Global Middleware
* **Rate Limiting:** `GovernorLayer` (2 req/sec, burst 5) via client IP.
* **Timeouts:** `TimeoutLayer` (10-second limit).
* **Observability:** `TraceLayer` structured metrics.

### Routes Matrix

| Method | Endpoint | Description | Extractors / Middleware |
| :--- | :--- | :--- | :--- |
| **GET** | `/` | Root Index | None |
| **GET** | `/pages` | Query-driven list pagination | `Query<Pagination>` |
| **POST**| `/login` | Authenticate and issue JWT | `Json<AuthPayload>` |
| **GET** | `/users/` | User section about | Concurrency Limited (Max 5) |
| **POST**| `/users/create` | Validate and insert new user | `Json<CreateUser>` + Concurrency Limited |
| **PATCH**| `/users/update/{id}` | Update user profile | **JWT (`Claims`)** + `Path<u64>` + `Json<UpdateUser>` + Concurrency Limited |
| **DELETE**| `/users/delete/{id}`| Remove a user by ID | **JWT (`Claims`)** + `Path<u64>` + Concurrency Limited |
| **GET** | `/users/greet/{name}`| Dynamic path injection | `Path<String>` + Concurrency Limited |
| **GET** | `/admin/list` | Fetch all users | **JWT (`Claims`)** + `Query<Pagination>` + Admin Role |
| **PATCH**| `/admin/{id}/role` | Modify user role level | **JWT (`Claims`)** + `Path<u64>` + `Json<ChangeRolePayload>` + Admin Role |
| **ANY** | `/assets/*` | Static asset / SPA fallback | `ServeDir` / `ServeFile` ("public") |

---

## Getting Started

### 1. Setup Environment
```env
JWT_SECRET=your_super_secret_key_here
```

### 2. Run the Server
```console
# Automatically builds SQLite schema and listens at http://localhost:3000
cargo run
```
### 3. Run Test
```console
cargo test
```
