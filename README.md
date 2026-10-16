<h1 align="center">Big Sky</h1>

<h3 align="center">The best web framework in the west.</h3>

<p align="center">
  A Nuxt frontend and a Rust API, wired together and working out of the box.
</p>

<p align="center">
  <img src="horizon/public/header.png" alt="Big Sky" width="720">
</p>

## What You Get

Clone it, start two services, and there is a working full-stack app on screen — not a placeholder page.

The landing page is a live demo of the wiring. It reads the current population from the Axum API on load, and the two buttons call real endpoints that mutate server state and return it, with a toast on each change. Every piece a real app needs is already threaded through: typed responses, CORS, SSR-safe hydration, and error-free reloads.

## The Stack

Three layers, each in its own top-level directory.

| Name | Layer | Built with | Port |
| --- | --- | --- | --- |
| **Horizon** | Frontend | Vue & Nuxt | `:3000` |
| **Bedrock** | Backend | Rust & Axum | `:8080` |
| **Cellar** | Database | PostgreSQL & SQLx | `:5432` |

```
Browser  ->  horizon  ->  bedrock  ->  cellar
             (Nuxt)      (Axum API)   (Postgres)
```

`horizon` renders the UI and talks to `bedrock` over HTTP. `bedrock` owns the business logic and is the only thing that touches the database. `cellar` stores the data, and nothing else reaches it directly.

## About the Names

The parts of a web app tend to come with jargon instead of names. These are a cross-section of the land itself, top to bottom.

- **Big Sky** — the whole thing. Everything above, below, and in between.
- **Horizon** — the frontend. The visible edge. What you actually look at.
- **Bedrock** — the backend. Structural, load-bearing, out of sight.
- **Cellar** — the database. Dug in, cool and dark, and it remembers everything.

## Getting Started

Requires Node.js 20+, pnpm 11+, and Rust 1.85+ (the floor for the 2024 edition).

```bash
git clone https://github.com/joncorv/big-sky.git
cd big-sky
```

Then start each layer in its own terminal.

```bash
cd horizon && pnpm install && pnpm dev    # http://localhost:3000
cd bedrock && cargo run                   # http://localhost:8080
```

## The API

| Route  | Returns                                    |
| ------ | ------------------------------------------ |
| `/`    | Current population                         |
| `/add` | Increments, returns the count and a message |
| `/sub` | Decrements, returns the count and a message |

## Persistence

`bedrock` currently holds the population in `AppState` behind an `Arc<Mutex<i32>>`, so the count is live and shared across clients but resets when the server restarts.

Swapping that for Postgres is the one outstanding piece. The seam is deliberately narrow — the three handlers in `bedrock/src/main.rs` are the only things that touch the state, so a `PgPool` in `AppState` and three queries is the whole job. `cellar/` is where that service definition goes.

## License

MIT. See [`horizon/LICENSE`](horizon/LICENSE); a root license file is still to be added.
