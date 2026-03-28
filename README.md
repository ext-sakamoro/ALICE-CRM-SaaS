# ALICE-CRM-SaaS

Customer Relationship Management API — contacts, deals, pipeline management, and AI-based lead scoring via the ALICE SaaS architecture.

## Architecture

```
Client
  └─ API Gateway (:8144) — JWT auth, rate limiting, proxy
       └─ Core Engine (:9144) — contact store, deal engine, pipeline, scorer
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | /health | Health check |
| GET/POST | /api/v1/crm/contacts | List or create contacts |
| GET/POST | /api/v1/crm/deals | List or create deals |
| GET | /api/v1/crm/pipeline | Sales pipeline view |
| POST | /api/v1/crm/score | AI lead scoring |
| GET | /api/v1/crm/stats | Request statistics |

## Quick Start

```bash
cd services/core-engine && cargo run
cd services/api-gateway && cargo run
```

## License

AGPL-3.0-or-later
