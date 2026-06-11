# Assignment

Task management API with JWT authentication, 2FA verification, and Redis caching.

## Prerequisites

- Rust (edition 2021)
- Docker & Docker Compose

## Quick Start

```bash
# 1. Start Postgres and Redis
docker compose up -d

# 2. Copy and configure environment
cp .env.example .env   # or create .env with DATABASE_URL and REDIS_URL

# 3. Run the server (migrations run automatically)
cargo run
```

Server starts at `http://0.0.0.0:3000`.

## Environment Variables

| Variable      | Default                                        | Description     |
|---------------|------------------------------------------------|-----------------|
| `DATABASE_URL` | `postgres://postgres:postgres@localhost:5432/assignment` | Postgres DSN |
| `REDIS_URL`   | `redis://127.0.0.1:6379`                      | Redis URL       |
| `JWT_SECRET`  | `default_secret_key_change_in_prod`            | JWT signing key |

## API Endpoints

| Method | Path                    | Auth Required | Description               |
|--------|-------------------------|---------------|---------------------------|
| GET    | `/ping`                 | No            | Health check              |
| POST   | `/seed/users`           | No            | Create admin + staff seed users |
| POST   | `/auth/login`           | No            | Login (returns 2FA code via email logs) |
| POST   | `/auth/verify-2fa`      | No            | Verify 2FA code, returns JWT |
| GET    | `/dev/email-logs/latest`| No            | Retrieve sent 2FA codes   |
| POST   | `/tasks`                | Admin         | Create a task             |
| POST   | `/tasks/assign`         | Admin         | Assign task to a user     |
| GET    | `/tasks/view-my-tasks`  | Any           | View assigned tasks       |

## User Roles

- **admin** — can create tasks, assign tasks to users
- **staff** — can only view their assigned tasks

### Seed Users

| Username    | Password  | Role  |
|-------------|-----------|-------|
| `admin`     | `admin123`| admin |
| `james_bond`| `bond123` | staff |

## Testing

```bash
# Run unit tests only (no external services needed for compilation)
cargo check --tests

# Run integration tests (requires docker compose up)
cargo test -- --ignored --test-threads=1
```
