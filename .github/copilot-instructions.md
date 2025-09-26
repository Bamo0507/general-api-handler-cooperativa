# Copilot Instructions for AI Coding Agents

## Project Overview
This is a Rust backend for a cooperative, handling authentication, payments, loans, fines, and quotas. It exposes both REST and GraphQL endpoints. Redis is used for caching and session management. The project is containerized using Docker/Podman.

## Architecture & Key Components
- **src/**: Main source code. Organized by domain (models, repos, endpoints).
  - `models/`: Data structures for auth, payments, loans, fines, quotas, and Redis.
  - `repos/`: Data access logic, split by domain (auth, graphql, etc.).
  - `endpoints/`: API endpoints, split into REST and GraphQL handlers.
- **config.rs**: Configuration loading and environment setup.
- **main.rs**: Application entry point.
- **Dockerfile / docker-compose**: Containerization and local development setup.

## Developer Workflows
- **Build**: `cargo build`
- **Test**: `cargo test` (unit/integration tests in `tests/`)
- **Run Locally**: Use `docker-compose up --build` or `podman-compose up --build` for full environment.
- **Debug**: Use Rust's standard debugging tools. For container debugging, attach to running containers.

## Project-Specific Conventions
- **GraphQL**: Handlers and resolvers are in `src/endpoints/handlers/graphql/` and `src/repos/graphql/`.
- **REST**: Handlers in `src/endpoints/handlers/rest/`.
- **Auth**: Managed via `models/auth.rs` and `repos/auth/`.
- **Redis**: Used for session and cache, see `models/redis.rs`.
- **Testing**: Tests are organized by domain in `tests/`, e.g., `tests/graphql/` for GraphQL logic.
- **Spanish Naming**: Many files, variables, and comments use Spanish (e.g., `cuota`, `prestamo`, `afiliado`).

## Integration Points
- **External Services**: Redis, Docker/Podman containers.
- **Internal Communication**: Data flows from endpoints → models → repos.

## Examples
- To add a new GraphQL resolver, update `src/repos/graphql/` and register in `src/endpoints/handlers/graphql/`.
- For a new REST endpoint, add handler in `src/endpoints/handlers/rest/` and update routing logic.

## Additional Notes
- Only run tests in a development environment, not production.
- The compose file is versioned for now, but may change.

---

For questions about conventions or unclear patterns, review `README.md` and domain-specific files in `src/`.
