# Dororo

An asynchronous backend sandbox built to learn how to build a web services in Rust.

---

## Core Learning Takeaways

* **Thread-Safe State & Static Keys:** Sharing DB pools via `Arc<AppState>` and lazy initializing crypto keys with `std::sync::LazyLock`.
* **Type-Safe Extraction & RBAC:** Extracting Bearer tokens via `TypedHeader` into `Claims` paired with role-matching middleware (`require_role`).
* **Socketless Integration Testing:** Testing the HTTP pipeline in-memory via `tower::Service` (`oneshot`/`call`) using an ephemeral `sqlite::memory:` database.
* **Defensive Traffic Control:** Layering Tower middleware for global rate-limiting (`GovernorLayer`) and timeouts (`TimeoutLayer`).
* **Declarative Payload Validation:** Binding the `validator` crate to deserialization pipelines to sanitize input data before hitting domain logic.

---

## API Endpoints

### Global Middleware
* **Rate Limiting:** `GovernorLayer` (50 req/sec, burst 200) via client IP.
* **Timeouts:** `TimeoutLayer` (10-second limit).
* **Observability:** `TraceLayer` structured metrics.

### Routes Matrix

| Method | Endpoint | Description | Extractors / Middleware |
| :--- | :--- | :--- | :--- |
| **GET** | `/` | Root Index | None |
| **GET** | `/pages` | Query-driven list pagination | `Query<Pagination>` |
| **POST**| `/login` | Authenticate and issue JWT | `Json<AuthPayload>` |
| **GET** | `/users/` | User section about | None |
| **POST**| `/users/create` | Validate and insert new user | `Json<CreateUser>` |
| **PATCH**| `/users/update/{id}` | Update user profile | **JWT (`Claims`)** + `Path<u64>` + `Json<UpdateUser>` |
| **DELETE**| `/users/delete/{id}`| Remove a user by ID | **JWT (`Claims`)** + `Path<u64>` |
| **GET** | `/users/greet/{name}`| Dynamic path injection | `Path<String>` |
| **GET** | `/admin/list` | Fetch all users | **JWT (`Claims`)** + `Query<Pagination>` + Admin Role |
| **PATCH**| `/admin/{id}/role` | Modify user role level | **JWT (`Claims`)** + `Path<u64>` + `Json<ChangeRolePayload>` + Admin Role |
| **POST**| `/owner/transfer-ownership` | Transfer company ownership | **JWT (`Claims`)** + `Json<TransferOwnershipPayload>` + Owner Role |
| **PATCH**| `/owner/rename-company` | Update company name for all members | **JWT (`Claims`)** + `Json<UpdateCompanyPayload>` + Owner Role |
| **ANY** | `/assets/*` | Static asset / SPA fallback | `ServeDir` / `ServeFile` ("public") |

---

## Getting Started

```console
# copy env
cp env.example env

# run the service
docker compose up --build

# testing
cargo test
```
