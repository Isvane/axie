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

* **Axum and Tokio:** Handles the HTTP routes and async runtime.
* **Non-blocking Auth:** Argon2 password hashing takes time, so I threw it into tokio::task::spawn_blocking so it doesn't freeze the main async event loop while users log in.
* **Role-Based Access Control:** Custom JWT extractor that guards routes based on roles (User, Admin, and Owner).
* **Jemalloc Allocator:** Swapped the default memory allocator for tikv-jemallocator to bypass musl's allocation bottlenecks on Alpine.
* **Database and Migration:** Uses toasty ORM with embedded migrations that run automatically when the app boots up.
* **Traffic Guards:** Added rate limiting (tower-governor) so endpoints don't get spammed, plus graceful shutdowns when terminating the process.

---

## API Routes

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
