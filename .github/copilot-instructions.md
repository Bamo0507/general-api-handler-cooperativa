# Copilot Instructions for AI Agents

## Project Overview
- Rust backend for a cooperative, using Actix Web, Juniper (GraphQL), Redis, and Docker/Podman.
- Main service boundaries: authentication, affiliate quotas, loan quotas, fines, payments.
- Data flows: Redis stores all quota/payment data; GraphQL endpoints expose queries/mutations; REST endpoints for auth.

## Key Workflows
- **Build/Run:**
  - Use `docker-compose up --build` or `podman-compose up --build` to start all services.
  - Local dev: `cargo run` (main binary: `general-api`)
- **Testing:**
  - Run all tests: `cargo test -- --nocapture`
  - Test specific modules: e.g. `cargo test --test cuota_afiliado_test -- --nocapture`
  - Redis is required for tests; tests clean up Redis state per test.
- **Dev Deployment:**
  - GitHub Actions workflow in `.github/workflows/dev-deployment.yml` auto-deploys on `dev-deployment` branch push.

## Architectural Patterns
- **Models:** Defined in `src/models/graphql.rs` (see `Quota`, `TipoCuota`, `Affiliate`, etc.)
- **Resolvers:** GraphQL resolvers in `src/endpoints/handlers/graphql/` (see `Quota.rs` for quota logic)
- **Repositories:** Data access in `src/repos/graphql/` (see `Quota.rs` for Redis logic)
- **Strict Response Formats:**
  - All quota responses must match `docs/api-quota-response-format.md` exactly (field names, types, enums, nullability).
  - Example: Affiliate quota identifier is `"Nombre - Mes Año"` in Spanish.
- **Authentication:**
  - REST endpoints: `/general/signup`, `/general/login` (returns access token)
  - GraphQL: Pass access token as `user_id` parameter in queries; response `user_id` must echo the token.

## Project-Specific Conventions
- **Spanish field names** in models for easier integration with frontend.
- **Redis key patterns:**
  - Affiliate quotas: `users:{access_token}:cuotas_afiliado:{fecha}`
  - Loan quotas: `users:{access_token}:loans:{loan_id}:cuotas:{fecha}`
- **Testing:**
  - Each test cleans Redis state for isolation.
  - Use helpers in test files for context/repo setup.
- **GraphQL Endpoints:**
  - Register resolvers in `src/endpoints/graphql_endpoints.rs` (see `get_cuotas_afiliado_mensuales_formateadas`, `get_cuotas_prestamo_pendientes_formateadas`).
  - Always update schema and endpoint registration when adding new resolvers.

## Integration Points
- **External:**
  - Redis (local or cloud, configure via `REDIS_URL` env var)
  - Docker/Podman for container orchestration
- **Internal:**
  - All business logic for quotas/payments is in `src/repos/graphql/` and `src/endpoints/handlers/graphql/`

## Examples
- **Affiliate Quota Response:**
  ```json
  {
    "identifier": "Maria Lopez - Agosto 2025",
    "user_id": "token access",
    "monto": 150.0,
    "nombre": "Maria Lopez",
    "fecha_vencimiento": "YY-MM-DD",
    "extraordinaria": false
  }
  ```
- **Loan Quota Response:**
  ```json
  {
    "user_id": "access_token",
    "monto": 100.0,
    "fecha_vencimiento": "2025-01-05",
    "monto_pagado": 0.0,
    "multa": 0.0,
    "pagada_por": null,
    "tipo": "Prestamo",
    "loan_id": "loan_abc",
    "pagada": false,
    "numero_cuota": 1,
    "nombre_prestamo": "Personal Loan"
  }
  ```

## References
- `docs/api-quota-response-format.md`: Canonical response formats
- `src/models/graphql.rs`: Data models
- `src/endpoints/handlers/graphql/`: Resolver logic
- `src/repos/graphql/`: Data access patterns
- `tests/graphql/`: Test coverage and patterns
- `.github/workflows/dev-deployment.yml`: CI/CD workflow

---
_Last updated: September 22, 2025_
