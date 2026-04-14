# Adeptus

**Technical Documentation Management Backend Service**

Adeptus is a backend service for managing, serving, and generating technical documentation. It stores documents in [Markdoc](https://markdoc.dev/) format, exposes a GraphQL API for all operations, provides REST endpoints for file handling and PDF generation, and maintains a comprehensive audit trail of every action. Adeptus is one service in a larger enterprise platform --- it runs independently but integrates with shared authentication, media, and observability infrastructure when deployed alongside sibling services.

---

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Features](#features)
- [Technology Stack](#technology-stack)
- [Project Structure](#project-structure)
- [API Reference](#api-reference)
  - [GraphQL Queries](#graphql-queries)
  - [GraphQL Mutations](#graphql-mutations)
  - [GraphQL Subscriptions](#graphql-subscriptions)
  - [REST Endpoints](#rest-endpoints)
- [Database Schema](#database-schema)
- [Document Types](#document-types)
- [Content Format (Markdoc)](#content-format-markdoc)
- [Document Lifecycle](#document-lifecycle)
- [Services Layer](#services-layer)
- [Middleware](#middleware)
- [Configuration](#configuration)
- [Platform Integration](#platform-integration)
  - [Authentication (Ory Ecosystem)](#authentication-ory-ecosystem)
  - [Media and Asset Management](#media-and-asset-management)
  - [Sibling Services](#sibling-services)
  - [Frontend](#frontend)
  - [Observability](#observability)
- [Deployment](#deployment)
- [Development](#development)

---

## Architecture Overview

```
                                  Adeptus
 ┌──────────────────────────────────────────────────────────────────┐
 │                                                                  │
 │   Axum Router                                                    │
 │   ├── POST /graphql ─────────► async-graphql (Query, Mutation)   │
 │   ├── GET  /graphql ─────────► GraphiQL Playground               │
 │   ├── WS   /graphql ─────────► Subscriptions (placeholder)       │
 │   ├── GET  /api/files/:id/download ──► FileService               │
 │   ├── POST /api/files/upload ────────► FileService               │
 │   ├── GET  /api/documents/:id/pdf ──► PdfService                 │
 │   ├── GET  /api/documents/:id/version ► PdfService               │
 │   └── GET  /health, /ready, /metrics ► HealthChecks              │
 │                                                                  │
 │   Middleware Stack (Tower)                                       │
 │   ├── CORS                                                       │
 │   ├── Rate Limiting (governor)                                   │
 │   ├── Request Tracing (OpenTelemetry + Jaeger)                   │
 │   ├── Request ID injection                                       │
 │   ├── JWT Authentication (optional per route)                    │
 │   └── Audit Logging                                              │
 │                                                                  │
 │   Services                                                       │
 │   ├── DocumentService ─── CRUD, revisions, relationships         │
 │   ├── GlossaryService ─── entries, search, categories            │
 │   ├── FileService ────── upload, download, CDN integration       │
 │   └── PdfService ─────── Markdoc→HTML→PDF via wkhtmltopdf        │
 │                                                                  │
 │   Database Layer                                                 │
 │   └── PostgreSQL (sqlx, async, BB8 connection pool)              │
 │                                                                  │
 └──────────────────────────────────────────────────────────────────┘
           │                    │                      │
           ▼                    ▼                      ▼
      PostgreSQL          Ory Ecosystem           CDN / DAM
      (9 tables)       (Kratos, Hydra,        (media storage,
                        Keto, Oathkeeper)      file delivery)
```

Adeptus is a single Rust crate (not a Cargo workspace), built with the 2021 edition.

---

## Features

- **GraphQL-first API** via async-graphql 7.0 with full introspection, federation support, and a built-in GraphiQL playground.
- **Document management** with create, read, update, delete, publish, unpublish, and archive operations.
- **Automatic revision tracking** --- every content change creates a numbered revision snapshot.
- **Inter-document relationships** --- link documents as related, supersedes, references, translation, or version.
- **Hierarchical categories** for both documents and glossary entries, with self-referencing parent IDs.
- **Glossary management** --- full CRUD for glossary entries with search, multi-language support, and relationship types (related, see_also, synonym, antonym, parent, child).
- **File handling** --- multipart upload with validation (100 MB max, extension whitelist, MIME type checking), local storage with optional CDN upload, and CDN-fallback downloads.
- **PDF generation** --- converts Markdoc to HTML, then to PDF via wkhtmltopdf. Optionally injects JavaScript that checks a version endpoint to warn readers when a newer revision exists. Supports watermarked PDF output with the downloader's name and email visibly stamped on each page, plus a hidden metadata download ID embedded in the PDF for traceability.
- **Comprehensive audit logging** --- every entity operation (create, update, delete, view, upload, download, publish, unpublish, archive) is recorded with user ID, IP address, user agent, and a JSONB diff of changes.
- **No PII storage** --- audit logs reference opaque subject identifiers. Identity management is fully delegated to the Ory ecosystem.
- **Multi-language support** --- documents and glossary entries carry an ISO 639-1 language code; relationships can express translation links.
- **JWT authentication** with scope-based authorisation (read, write, admin, publish) and group-based admin detection.
- **Distributed tracing** via OpenTelemetry with Jaeger export, JSON structured logging, and custom spans for database, service, file, and GraphQL operations.
- **API rate limiting** via the `governor` crate with configurable global, per-user, and per-IP limits and burst allowance.
- **Health, readiness, and metrics endpoints** for Kubernetes-native deployments.
- **Graceful shutdown** with SIGTERM/SIGINT handling and tracer flush.

---

## Technology Stack

| Concern | Technology | Version / Notes |
|---|---|---|
| Language | Rust | 2024 edition |
| Web framework | Axum | 0.8, with Tower middleware |
| GraphQL | async-graphql | 7.2 (UUID, chrono features) |
| Database | PostgreSQL | via sqlx 0.7 (compile-time query checking) |
| Async DB driver | sqlx | 0.7 (postgres, runtime-tokio features) |
| Connection pool | BB8 | 0.9 |
| Authentication | jsonwebtoken | 10.3 (HS256) |
| HTTP client | reqwest | 0.13 (JSON, multipart) |
| PDF generation | wkhtmltopdf | 0.4 crate + system binary |
| Tracing | tracing + tracing-subscriber | JSON output, env-filter |
| OpenTelemetry | opentelemetry + opentelemetry-jaeger | 0.31 / 0.22 |
| Serialisation | serde + serde_json | 1.0 |
| Date/Time | chrono | 0.4 |
| UUIDs | uuid | 1.20 (v4) |
| Error handling | anyhow + thiserror | 1.0 |
| Configuration | config + dotenvy | 0.14 / 0.15 |
| Temp files | tempfile | 3.24 |
| MIME | mime + mime_guess | 0.3 / 2.0 |
| Regex | regex | 1.12 |
| HTML escaping | html-escape | 0.2 |

---

## Project Structure

```
adeptus/
├── Cargo.toml                         # Single crate manifest (Rust 2021)
├── Cargo.lock
├── sqlx.toml                          # sqlx CLI configuration
├── README.md                          # This file
├── migrations/
│   └── 2024010100000_create_initial_schema/
│       └── up.sql / down.sql          # All 9 tables, indexes, triggers, seed data
├── src/
│   ├── main.rs                        # Entry point: config load, server start, shutdown
│   ├── lib.rs                         # AppState, router, server setup, meta module
│   ├── schema.rs                      # sqlx query definitions (9 tables)
│   ├── config/
│   │   └── mod.rs                     # Config struct, env loading, defaults, validation
│   ├── models/
│   │   ├── mod.rs                     # DbPool type alias, PaginationParams
│   │   ├── document.rs                # Document, Category, Revision, Relationship, File models + GQL types + inputs
│   │   ├── glossary.rs                # GlossaryEntry, Category, Relationship models + GQL types + inputs
│   │   └── audit.rs                   # AuditLog, AuditContext, EntityType, AuditAction, Auditable trait
│   ├── graphql/
│   │   ├── mod.rs                     # Schema builder, context, error extensions, scalars, validation, directives
│   │   ├── query.rs                   # All GraphQL queries (documents, glossary, audit, health, system)
│   │   ├── mutation.rs                # All GraphQL mutations (documents, glossary, files, PDF)
│   │   └── subscription.rs            # Placeholder subscriptions (document, glossary, system)
│   ├── services/
│   │   ├── mod.rs                     # AppServices container, ServiceError enum, ServiceResult
│   │   ├── document.rs                # DocumentService: CRUD, revisions, relationships, publishing, categories
│   │   ├── glossary.rs                # GlossaryService: entries, search, categories, relationships
│   │   ├── file.rs                    # FileService: upload, download, delete, CDN integration, validation
│   │   └── pdf.rs                     # PdfService: Markdoc→HTML→PDF, version-check JS injection
│   ├── handlers/
│   │   ├── mod.rs                     # Error responses, request helpers, pagination, search validation
│   │   ├── graphql.rs                 # GraphQL POST handler, playground, WebSocket subscriptions, schema SDL
│   │   ├── health.rs                  # /health, /ready, /metrics, liveness, system status
│   │   ├── file.rs                    # File download/upload REST handlers
│   │   └── pdf.rs                     # PDF generation and version-check REST handlers
│   └── middleware/
│       ├── mod.rs                     # Middleware stack, CORS config, trace layer, request ID, IP extraction
│       ├── auth.rs                    # JWT validation (HS256), Claims, AuthContext, scope/group checking
│       ├── audit.rs                   # Request logging, AuditService (entity ops, file ops, lifecycle)
│       └── tracing.rs                 # OpenTelemetry + Jaeger init, custom spans, metrics helpers, error logging
└── target/                            # Build artifacts
```

---

## API Reference

### GraphQL Queries

All queries are available via `POST /graphql`. An interactive GraphiQL playground is served at `GET /graphql`.

#### Documents

| Query | Arguments | Returns | Auth |
|---|---|---|---|
| `document` | `id: ID!` | `DocumentGql?` | Optional |
| `documents` | `filter: DocumentFilterInput, limit: Int, offset: Int` | `[DocumentGql]` | Optional |
| `documentRevisions` | `documentId: ID!` | `[DocumentRevisionGql]` | Optional |
| `documentRevision` | `documentId: ID!, revisionNumber: Int!` | `DocumentRevisionGql?` | Optional |
| `documentRelationships` | `documentId: ID!` | `[DocumentRelationshipGql]` | Optional |
| `documentCategories` | -- | `[DocumentCategoryGql]` | Optional |
| `documentCategory` | `id: ID!` | `DocumentCategoryGql?` | Optional |
| `documentFiles` | `documentId: ID!` | `[DocumentFileGql]` | Optional |
| `file` | `id: ID!` | `DocumentFileGql?` | Optional |

**DocumentFilterInput** fields: `language`, `status` (Draft/Published/Archived/Unpublished), `categoryId`, `tags`, `search`, `createdBy`, `dateFrom`, `dateTo`.

#### Glossary

| Query | Arguments | Returns | Auth |
|---|---|---|---|
| `glossaryEntry` | `id: ID!` | `GlossaryEntryGql?` | Optional |
| `glossaryEntries` | `filter: GlossaryFilterInput, limit: Int, offset: Int` | `[GlossaryEntryGql]` | Optional |
| `searchGlossary` | `searchTerm: String!, language: String` | `[GlossaryEntryGql]` | Optional |
| `glossaryEntryRelationships` | `entryId: ID!` | `[GlossaryRelationshipGql]` | Optional |
| `relatedGlossaryEntries` | `entryId: ID!` | `[GlossaryEntryGql]` | Optional |
| `glossaryCategories` | -- | `[GlossaryCategoryGql]` | Optional |
| `glossaryCategory` | `id: ID!` | `GlossaryCategoryGql?` | Optional |
| `glossaryEntriesByCategory` | `categoryId: ID!` | `[GlossaryEntryGql]` | Optional |
| `glossaryEntriesByLanguage` | `language: String!` | `[GlossaryEntryGql]` | Optional |

**GlossaryFilterInput** fields: `language`, `categoryId`, `search`, `createdBy`.

#### Administration

| Query | Arguments | Returns | Auth |
|---|---|---|---|
| `auditLogs` | `filter: AuditLogFilterInput, limit: Int, offset: Int` | `[AuditLogGql]` | Admin required |
| `systemInfo` | -- | `JSON` | Admin required |
| `health` | -- | `String` | None |
| `documentVersion` | `id: ID!, currentVersion: Int` | `JSON` | Optional |

**AuditLogFilterInput** fields: `entityType`, `entityId`, `action`, `userId`, `dateFrom`, `dateTo`.

### GraphQL Mutations

All mutations require JWT authentication. The required scope is noted per operation.

#### Document Mutations

| Mutation | Arguments | Returns | Scope |
|---|---|---|---|
| `createDocument` | `input: CreateDocumentInput!` | `DocumentGql` | `write` |
| `updateDocument` | `id: ID!, input: UpdateDocumentInput!` | `DocumentGql?` | `write` |
| `deleteDocument` | `id: ID!` | `Boolean` | `write` |
| `publishDocument` | `id: ID!` | `DocumentGql?` | `publish` or admin |
| `unpublishDocument` | `id: ID!` | `DocumentGql?` | `publish` or admin |
| `archiveDocument` | `id: ID!` | `DocumentGql?` | `write` |
| `createDocumentCategory` | `input: CreateDocumentCategoryInput!` | `DocumentCategoryGql` | `admin` |
| `createDocumentRelationship` | `input: CreateDocumentRelationshipInput!` | `DocumentRelationshipGql` | `write` |

**CreateDocumentInput** fields: `title` (required), `content` (required), `description`, `language` (default `"en"`), `categoryId`, `tags`, `status` (default Draft), `publishDate`.

**UpdateDocumentInput** fields (all optional): `title`, `description`, `content`, `language`, `categoryId`, `tags`, `status`, `publishDate`, `changesSummary`. When `content` changes, a new revision is automatically created.

#### File Mutations

| Mutation | Arguments | Returns | Scope |
|---|---|---|---|
| `uploadFiles` | `documentId: ID!, files: [Upload!]!` | `[DocumentFileGql]` | `write` |
| `deleteFile` | `id: ID!` | `Boolean` | `write` |

#### Glossary Mutations

| Mutation | Arguments | Returns | Scope |
|---|---|---|---|
| `createGlossaryEntry` | `input: CreateGlossaryEntryInput!` | `GlossaryEntryGql` | `write` |
| `updateGlossaryEntry` | `id: ID!, input: UpdateGlossaryEntryInput!` | `GlossaryEntryGql?` | `write` |
| `deleteGlossaryEntry` | `id: ID!` | `Boolean` | `write` |
| `createGlossaryCategory` | `input: CreateGlossaryCategoryInput!` | `GlossaryCategoryGql` | `admin` |
| `updateGlossaryCategory` | `id: ID!, input: UpdateGlossaryCategoryInput!` | `GlossaryCategoryGql?` | `admin` |
| `deleteGlossaryCategory` | `id: ID!` | `Boolean` | `admin` |
| `createGlossaryRelationship` | `input: CreateGlossaryRelationshipInput!` | `GlossaryRelationshipGql` | `write` |
| `deleteGlossaryRelationship` | `id: ID!` | `Boolean` | `write` |

#### PDF Mutation

| Mutation | Arguments | Returns | Scope |
|---|---|---|---|
| `generatePdf` | `documentId: ID!, versionCheck: Boolean, watermark: Boolean` | `String` (base64 PDF) | `read` |

### GraphQL Subscriptions

Subscriptions are available over WebSocket at `WS /graphql`. The current implementation provides placeholders for:

- `documentUpdates` --- notifies subscribers of document changes.
- `glossaryUpdates` --- notifies subscribers of glossary changes.
- `systemNotifications` --- system-wide notification stream.

These are designed to be backed by a message broker (e.g. Redis pub/sub) in production.

### REST Endpoints

| Method | Path | Description | Auth |
|---|---|---|---|
| `POST` | `/graphql` | GraphQL queries and mutations | Per-operation |
| `GET` | `/graphql` | GraphiQL interactive playground | None |
| `WS` | `/graphql` | GraphQL subscriptions (WebSocket) | Per-operation |
| `GET` | `/api/files/:id/download` | Download a file by ID | Required |
| `POST` | `/api/files/upload` | Upload files (multipart/form-data) | Required (`write`) |
| `GET` | `/api/documents/:id/pdf` | Generate and download PDF for a document | Required |
| `GET` | `/api/documents/:id/version` | Get latest version info (used by PDF JS) | None |
| `GET` | `/health` | Basic liveness check | None |
| `GET` | `/ready` | Readiness check (DB + PDF service) | None |
| `GET` | `/metrics` | Prometheus-format metrics | None |

---

## Database Schema

Adeptus uses PostgreSQL with the `uuid-ossp` extension. The schema consists of 9 tables managed via sqlx migrations.

### Entity-Relationship Diagram

```
 document_categories ◄─────────────────────── documents
   id (PK, UUID)                                id (PK, UUID)
   name                                         title
   description                                  description
   parent_id (FK → self) ◄── hierarchy          content (Markdoc)
   created_at                                   language
   updated_at                                   category_id (FK)
                                                tags (TEXT[])
                                                status (draft|published|archived|unpublished)
                                                upload_date
                                                publish_date
                                                created_by (opaque subject ID)
                                                created_at
                                                updated_at
                                                    │
                              ┌──────────────────┬──┴──────────────────┐
                              ▼                  ▼                     ▼
                    document_revisions   document_relationships   document_files
                      id (PK, UUID)        id (PK, UUID)           id (PK, UUID)
                      document_id (FK)     source_document_id(FK)  document_id (FK)
                      revision_number      target_document_id(FK)  filename
                      title                relationship_type       original_filename
                      description            (related, supersedes,  file_path
                      content (snapshot)      references,           cdn_url
                      changes_summary         translation, version) mime_type
                      created_by           created_at              file_size
                      created_at                                   uploaded_by
                      UNIQUE(doc_id,rev#)                          created_at


 glossary_categories ◄───────────────────── glossary_entries ──────► glossary_relationships
   id (PK, UUID)                              id (PK, UUID)           id (PK, UUID)
   name                                       title                   source_entry_id (FK)
   description                                description             target_entry_id (FK)
   parent_id (FK → self) ◄── hierarchy        language                relationship_type
   created_at                                 category_id (FK)          (related, see_also,
   updated_at                                 created_by                 synonym, antonym,
                                              created_at                 parent, child)
                                              updated_at               created_at
                                                                       UNIQUE(source,target)


 audit_logs
   id (PK, UUID)
   entity_type (document, document_category, document_revision,
                document_file, glossary_entry, glossary_category, user, system)
   entity_id (UUID)
   action (create, update, delete, view, download, upload, publish, unpublish, archive)
   user_id (opaque subject ID, never PII)
   ip_address (INET, nullable)
   user_agent (TEXT, nullable)
   changes (JSONB, nullable --- stores diffs and operation metadata)
   created_at
```

### Indexes

The migration creates the following indexes for query performance:

- `documents`: language, status, category_id, publish_date, created_at, tags (GIN)
- `document_revisions`: document_id, (document_id + revision_number)
- `document_files`: document_id
- `glossary_entries`: language, category_id, title
- `audit_logs`: (entity_type + entity_id), user_id, created_at

### Unique Constraints

- `document_revisions`: (document_id, revision_number)
- `document_relationships`: (source_document_id, target_document_id, relationship_type)
- `glossary_relationships`: (source_entry_id, target_entry_id)

### Seed Data

The initial migration seeds default categories:

**Document categories:** Manuals, Guides, Release Notes, Service Notes, Safety Notes.

**Glossary categories:** Technical Terms, Business Terms, Safety Terms, General.

---

## Document Types

Adeptus manages the following document types, as specified in the `flux/` directory:

| Type | Description |
|---|---|
| **Release Items** | Atomic, referenceable units of change (bug fixes, features, improvements). The building block of release notes. |
| **Known Issues** | Documented issues with workarounds, severity, and affected version ranges. |
| **Release Notes** | Composed documents that reference a collection of release items and known issues for a software version. |
| **Manuals** | Technical documentation --- both atomic single-file manuals and composed multi-file manuals with table of contents. |
| **Service Notes** | Service and maintenance procedures, bulletins, and field instructions. |
| **Safety Notes** | Safety guidelines, hazard warnings, regulatory compliance documentation. |
| **Product Notes** | Product announcements --- new hardware, retirements, replacements, revisions. |
| **Guides** | How-to guides, tutorials, and procedural walkthroughs. |
| **FAQs** | Frequently asked questions organised by topic. |

The `flux/EXAMPLES/` directory provides sample Markdoc documents for each type.

---

## Content Format (Markdoc)

Adeptus stores document content as raw [Markdoc](https://markdoc.dev/) --- a Markdown superset with structured extensions. Key design decisions:

- **Raw storage, client-side rendering.** Adeptus stores the Markdoc source as-is in the `content` column. It does not pre-render to HTML. Frontends receive the raw Markdoc and render it locally.
- **Conditional content blocks.** Markdoc's tag system supports conditional rendering based on device type, user role, deployment context, and other runtime variables. This is only possible because rendering happens client-side.
- **PDF generation server-side.** When a PDF is requested, Adeptus performs a simplified Markdoc-to-HTML conversion internally and passes it to wkhtmltopdf. The conversion handles headings, paragraphs, lists, code blocks, bold, italic, inline code, and links.

The canonical format specification and all document type schemas are defined in the `flux/` directory at the repository root:

- `flux/CONCEPT.md` --- complete specification for all document types, metadata schemas, file management, conditional rendering, and ingestion pipeline.
- `flux/EXAMPLES/` --- sample `.markdoc` files for every document type.

---

## Document Lifecycle

Documents follow a five-state lifecycle:

```
  ┌───────┐      publish       ┌───────────┐
  │ Draft │ ─────────────────► │ Published │
  └───┬───┘                    └─────┬─────┘
      │         unpublish            │
      │ ◄────────────────────────────┘
      │                              │
      │         archive              │         archive
      └──────────────────┐     ┌─────┘
                         ▼     ▼
                      ┌──────────┐
                      │ Archived │
                      └─────┬────┘
                            │         unpublish (retire)
                            ▼
                      ┌─────────────┐
                      │ Unpublished │
                      └─────────────┘
```

- **Draft** --- initial state. Document is being authored or revised. Not visible to end users.
- **Published** --- document is live and visible. Publishing sets the `publish_date` and requires the `publish` scope or admin privileges.
- **Archived** --- document is retired. Still queryable but marked as no longer current.
- **Unpublished** --- document is fully retired and no longer queryable by end users. This is the terminal state for documents that have been permanently withdrawn. Only accessible to administrators for audit purposes.

Unpublishing a published document reverts it back to draft status. Unpublishing an archived document transitions it to the Unpublished state, removing it from all query results.

---

## Services Layer

The `AppServices` struct holds all four services, initialised at startup and shared across handlers via Axum's state mechanism.

### DocumentService

- **CRUD**: create, get, list (with filtering and pagination), update, delete.
- **Automatic revisions**: when a document's content changes during an update, a new `document_revisions` row is created with an incremented revision number and a snapshot of the content. The first revision is created on document creation.
- **Relationships**: create and query inter-document relationships (related, supersedes, references, translation, version).
- **Publishing workflow**: publish, unpublish, archive --- each operation updates the document status and records an audit log.
- **Category management**: create and list hierarchical document categories.

### GlossaryService

- **Entry management**: full CRUD for glossary entries with multi-language support.
- **Search**: search entries by term and optional language filter.
- **Categories**: hierarchical categories with CRUD operations (admin only for create/update/delete).
- **Relationships**: create, delete, and query relationships between entries (related, see_also, synonym, antonym, parent, child).
- **Filtered queries**: by language, by category, by creator.

### FileService

- **Multipart upload**: accepts multipart/form-data uploads and associates files with documents.
- **Validation**: 100 MB maximum file size, extension whitelist (pdf, doc, docx, txt, md, html, htm, jpg, jpeg, png, gif, svg, webp, mp4, avi, mov, wmv, flv, zip, tar, gz, rar, json, xml, csv, xlsx, xls), MIME type checking.
- **Local + CDN storage**: files are always saved locally first. If CDN is enabled, the file is also uploaded to the CDN. The CDN URL is stored alongside the local path.
- **Download with CDN fallback**: downloads attempt the CDN URL first; if that fails, falls back to local storage.
- **Deletion**: removes from database, CDN (if present), and local filesystem.
- **Audit**: upload and download operations are individually logged.

### PdfService

- **Markdoc-to-HTML conversion**: a built-in converter handles headings (h1-h6), paragraphs, ordered and unordered lists, code blocks with language classes, bold, italic, inline code, and links. In production this would be replaced by a full Markdoc parser.
- **HTML-to-PDF**: uses wkhtmltopdf as an external process with A4 page size, 20mm margins, UTF-8 encoding, and JavaScript enabled.
- **Version-check JavaScript injection**: when enabled, injects a `<script>` block into the HTML before PDF generation. This script periodically calls the `/api/documents/{id}/version` endpoint and displays a banner if the PDF is outdated.
- **Watermarked PDF generation**: generates a personalised PDF watermarked with the downloader's name and email on each page (semi-transparent diagonal text overlay). A hidden metadata download ID is embedded in the PDF document properties for traceability --- if a watermarked PDF is leaked, the download ID can be traced back to the specific download event in the audit log.
- **Timeout**: configurable generation timeout (default 120 seconds).
- **Requirements validation**: checks that the wkhtmltopdf binary is present and functional at startup.
- **Custom options**: supports configurable page size, orientation, margins, JavaScript delay, and other wkhtmltopdf flags.

---

## Middleware

The middleware stack is built with Tower and applied to all routes in the following order:

### Rate Limiting

API rate limiting is powered by the `governor` crate with three tiers:

- **Global limit** — configurable maximum requests per minute across all clients (default 10,000/min).
- **Per-user limit** — configurable maximum requests per authenticated user (default 1,000/min).
- **Per-IP limit** — configurable maximum requests per IP address for unauthenticated traffic (default 100/min).

Each tier supports a configurable burst allowance (default 10). When a limit is exceeded, the service returns HTTP 429 (Too Many Requests) with a `Retry-After` header.

### CORS

Configurable allowed origins (defaults to `http://localhost:3000`), methods (GET, POST, PUT, DELETE, OPTIONS), and headers (Content-Type, Authorization, Accept, X-Request-ID). Supports wildcard (`*`) origin for development.

### Request Tracing (Tower-HTTP)

Every HTTP request gets a tracing span with method, URI, and version. Logs request start, response status with latency, and failure details. Integrates with the OpenTelemetry layer for distributed trace propagation.

### Request ID

Extracts `X-Request-ID` from incoming headers or generates a UUID v4. Attaches to the tracing span and echoes back in the response headers.

### Authentication (JWT)

Two modes:

- **`optional_auth_middleware`** (applied globally): extracts and validates the JWT if an `Authorization: Bearer <token>` header is present. If valid, an `AuthContext` is inserted into the request extensions. If absent or invalid, the request proceeds without authentication.
- **`auth_middleware`** (available for route-specific use): requires a valid JWT; returns 401 if missing or invalid.

JWT validation uses HS256 with the configured secret. Claims include `sub` (subject/user ID), `name`, `email`, `scopes` (array), `groups` (array), `exp`, `iat`, and `iss`. The issuer is validated against the configured identity provider URL.

The `AuthContext` provides helper methods:

- `has_scope(scope)` --- check a single scope (read, write, admin, publish).
- `has_any_scope(scopes)` / `has_all_scopes(scopes)` --- set operations.
- `is_in_group(group)` --- group membership check.
- `is_admin()` --- true if the user has the `admin` scope or belongs to the `admins` group.

### Audit Logging

The audit middleware fires asynchronously (fire-and-forget) after each request, logging the HTTP method, URI, status code, duration, user ID (or "anonymous"), IP address, and user agent to the `audit_logs` table.

The `AuditService` provides structured entity-level logging:

- `log_create`, `log_update`, `log_delete`, `log_view` --- generic entity operations.
- `log_file_upload`, `log_file_download` --- file-specific operations.
- `log_document_publish`, `log_document_unpublish`, `log_document_archive` --- lifecycle operations.
- `get_audit_logs` --- query with optional filters (entity type, entity ID, user ID) and pagination.

---

## Configuration

Configuration is loaded from environment variables, optional config files (`config/default`, `config/local`), and environment-prefixed overrides (`ADEPTUS__*`).

### Required Environment Variables

| Variable | Description |
|---|---|
| `DATABASE_URL` | PostgreSQL connection string (e.g. `postgresql://user:pass@localhost/adeptus`) |
| `JWT_SECRET` | Secret key for JWT validation. **Must be at least 32 characters.** |
| `JAEGER_ENDPOINT` | Jaeger agent endpoint for trace export (e.g. `http://localhost:14268/api/traces`) |

Note: The source code also reads an `OPENCONNECT_URL` environment variable for the JWT issuer validation. In the platform architecture, this corresponds to the Ory Hydra issuer URL.

### Optional Environment Variables

| Variable | Description |
|---|---|
| `CDN_API_KEY` | API key for CDN upload authentication |

### Configuration Defaults

| Setting | Default | Description |
|---|---|---|
| `server.host` | `0.0.0.0` | Bind address |
| `server.port` | `3000` | Listen port (validated: 1024-65535) |
| `server.cors_origins` | `["http://localhost:3000"]` | Allowed CORS origins |
| `database.max_connections` | `10` | Maximum DB pool connections |
| `database.min_connections` | `1` | Minimum idle DB pool connections |
| `database.connection_timeout` | `30` | Pool connection timeout in seconds |
| `cdn.enabled` | `false` | Enable CDN file upload |
| `cdn.base_url` | `""` | CDN API base URL |
| `cdn.bucket_name` | `"adeptus-documents"` | CDN storage bucket |
| `cdn.upload_timeout` | `300` | CDN upload timeout in seconds |
| `pdf.wkhtmltopdf_path` | `/usr/bin/wkhtmltopdf` | Path to wkhtmltopdf binary |
| `pdf.temp_dir` | `/tmp` | Temporary directory for PDF generation |
| `pdf.version_check_endpoint` | `/api/documents/{id}/version` | Version check URL template |
| `pdf.generation_timeout` | `120` | PDF generation timeout in seconds |
| `telemetry.service_name` | `"adeptus"` | OpenTelemetry service name |
| `telemetry.environment` | `"development"` | Deployment environment label |
| `telemetry.log_level` | `"info"` | Log level (trace, debug, info, warn, error) |
| `auth.jwt_expiration` | `3600` | JWT expiration time in seconds |
| `auth.required_scopes` | `["read", "write"]` | Scopes required for basic access |

### Configuration Validation

At startup, the configuration is validated:

- Database URL must not be empty.
- JWT secret must be at least 32 characters.
- Server port must be between 1024 and 65535.
- Max connections must be >= min connections.
- At least one CORS origin must be configured.
- Service name must not be empty.
- Log level must be one of: trace, debug, info, warn, error.

---

## Platform Integration

Adeptus is designed to operate standalone or as part of the larger enterprise platform. This section describes how it integrates when deployed alongside the other services.

### Authentication (Ory Ecosystem)

In the platform architecture, authentication and authorisation are delegated to the [Ory](https://www.ory.sh/) ecosystem:

| Component | Role |
|---|---|
| **Ory Kratos** | Identity management --- login, registration, session management, account recovery. |
| **Ory Hydra** | OAuth2 / OpenID Connect provider --- token issuance, consent flows. |
| **Ory Keto** | Fine-grained permissions --- Zanzibar-style relation tuples for resource-level access control. |
| **Ory Oathkeeper** | Zero-trust API gateway --- request authentication and authorisation at the edge. |

The authentication flow:

1. A client authenticates via Ory Kratos and obtains an OAuth2 token from Ory Hydra.
2. Requests pass through Ory Oathkeeper, which validates the JWT and extracts the subject identifier.
3. Adeptus receives the authenticated request with a verified JWT containing subject ID, scopes, and groups.
4. For resource-level access, the service can query Ory Keto with the subject, relation, and object.
5. Adeptus returns data or a 403 based on the authorisation result.

Internally, Adeptus validates JWTs using HS256 and the configured secret. The JWT claims carry scopes (`read`, `write`, `admin`, `publish`) and groups (e.g. `admins`), which Adeptus uses for route-level authorisation. No user data, profiles, or PII is stored in Adeptus --- only opaque subject identifiers appear in `created_by` fields and audit logs.

### Media and Asset Management

Adeptus manages document metadata and file references but delegates heavy binary storage to external systems:

- **Document files** (images, diagrams, attachments) can be uploaded through Adeptus and stored locally and/or on a CDN. The `document_files` table stores both a local `file_path` and an optional `cdn_url`.
- In the full platform deployment, media (images, photos, videos, diagrams) referenced by documents is handed off to a **Digital Asset Management (DAM)** system and served through a **CDN / globally distributed blob storage**.
- The DAM handles ingestion, transcoding, thumbnailing, metadata extraction, tagging, and version management.
- Public content is served directly from the CDN. Restricted content uses time-limited signed URLs or single-use tokens.
- Adeptus stores references (DAM asset IDs, CDN URLs, content hashes, MIME types) rather than the binary content itself.

### Sibling Services

Adeptus operates alongside other services in the platform:

| Service | Purpose | Relationship to Adeptus |
|---|---|---|
| **Sparta** | Spare parts tracking | Spare parts may reference Adeptus documents (manuals, service notes) for installation or maintenance procedures. |
| **Artemis** | Software artifact distribution | Release notes in Adeptus correspond to software versions managed in Artemis. |
| **Genesis** | Unit lifecycle and traceability | Genesis references Adeptus document IDs to link applicable manuals, safety notes, and service bulletins to specific production units. |

There are no hard runtime dependencies between services. Each owns its own database and API. Cross-service references are resolved at query time.

### Frontend

A unified **Dioxus**-based web frontend integrates with whichever backends are deployed:

- The frontend receives **raw Markdoc** from Adeptus (not pre-rendered HTML) and renders it client-side.
- Client-side rendering enables **conditional content blocks** based on device type, user role, and context.
- The frontend handles authentication flows through Ory Kratos/Hydra and adapts its navigation based on which backends are available.

### Observability

Adeptus uses a layered observability stack:

| Layer | Technology | Details |
|---|---|---|
| **Distributed tracing** | OpenTelemetry + Jaeger | Request tracing with custom spans for DB operations, service calls, file operations, GraphQL resolvers, and external API calls. Trace context propagates across service boundaries. |
| **Structured logging** | tracing + tracing-subscriber | JSON-formatted logs with target, thread ID, file, and line number. Environment-filtered with per-crate directives (`adeptus=debug`, `axum=info`, `sqlx=warn`, etc.). |
| **Metrics** | Prometheus-format `/metrics` endpoint | Service info, uptime, request counts, connection pool status. |
| **Audit logging** | `audit_logs` table | Every mutation, file operation, and lifecycle transition is recorded with user ID, IP, user agent, and JSONB change details. |

Custom span helpers are provided in `middleware::tracing::spans`:

- `db_span(operation, table)` --- database operations.
- `service_span(service, operation)` --- business logic operations.
- `external_api_span(service, endpoint, method)` --- outbound HTTP calls.
- `file_operation_span(operation, filename)` --- file I/O.
- `document_span(operation, document_id)` --- document-specific operations.
- `graphql_span(operation_type, operation_name)` --- GraphQL resolver tracing.

---

## Deployment

### Prerequisites

- **Rust** (stable toolchain, 2024 edition)
- **PostgreSQL** (with `uuid-ossp` extension)
- **wkhtmltopdf** installed at `/usr/bin/wkhtmltopdf` (or configured path)
- **sqlx-cli** for running migrations: `cargo install sqlx-cli --features postgres`

### Running Locally

```bash
# 1. Start PostgreSQL (e.g. via Docker)
docker run -d --name adeptus-db \
  -e POSTGRES_DB=adeptus \
  -e POSTGRES_USER=adeptus \
  -e POSTGRES_PASSWORD=secret \
  -p 5432:5432 \
  docker.io/postgres:18

# 2. Run migrations
export DATABASE_URL="postgresql://adeptus:secret@localhost/adeptus"
sqlx migrate run

# 3. Set required environment variables
export JWT_SECRET="your-secret-key-at-least-32-characters-long"
export JAEGER_ENDPOINT="http://localhost:14268/api/traces"
export OPENCONNECT_URL="https://your-ory-hydra-instance.com"

# 4. Run the service
cargo run
```

The service starts on `0.0.0.0:3000` by default. Visit `http://localhost:3000/graphql` for the GraphiQL playground.

### Health Checks

- `GET /health` --- basic liveness. Returns `{"status": "healthy"}`.
- `GET /ready` --- readiness. Checks database connectivity and wkhtmltopdf availability. Returns 200 if all checks pass, 503 otherwise.
- `GET /metrics` --- Prometheus-format metrics text.

### Kubernetes

Adeptus is designed as a stateless application that scales horizontally:

- Use the `/health` endpoint for liveness probes.
- Use the `/ready` endpoint for readiness probes.
- Configure `DATABASE_URL`, `JWT_SECRET`, `JAEGER_ENDPOINT` via Kubernetes Secrets.
- Set `CDN_API_KEY` and other optional values via ConfigMap or Secrets.
- The service handles `SIGTERM` for graceful shutdown (flushes traces, closes connections).
- Scale replicas independently based on traffic; the service shares no in-process state.

### Docker

```dockerfile
FROM docker.io/rust:1.93-alpine AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    libpq5 \
    wkhtmltopdf \
    ca-certificates \
  && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/adeptus /usr/local/bin/
EXPOSE 3000
CMD ["adeptus"]
```

---

## Development

### Building

```bash
cargo build          # Debug build
cargo build --release  # Release build
```

### Running Tests

```bash
cargo test
```

### Database Migrations

```bash
# Run pending migrations
sqlx migrate run

# Revert the last migration
sqlx migrate revert

# Generate a new migration
sqlx migrate add <name>
```

### GraphQL Schema Introspection

With the server running, the full GraphQL SDL is available programmatically via the schema endpoint, and the interactive playground at `GET /graphql` provides documentation, autocompletion, and query testing.

### Code Organisation Conventions

- **Models** define sqlx `FromRow` structs and their async-graphql `SimpleObject`/`InputObject` counterparts. Conversions between them are implemented via `From` traits.
- **Services** contain business logic and database operations. Each service owns an `AuditService` instance for logging.
- **Handlers** are thin wrappers that extract state, delegate to services, and format responses.
- **Middleware** provides cross-cutting concerns (auth, audit, tracing) as Tower layers.
- **GraphQL resolvers** (query.rs, mutation.rs) extract services from context, validate permissions, and delegate to the service layer.

### Error Handling

The `ServiceError` enum covers all error categories:

| Variant | HTTP Status | Description |
|---|---|---|
| `Database` | 500 | sqlx/PostgreSQL errors |
| `Validation` | 400 | Input validation failures |
| `NotFound` | 404 | Entity not found |
| `Unauthorized` | 401 | Missing or invalid authentication |
| `Forbidden` | 403 | Insufficient permissions |
| `Conflict` | 409 | Duplicate or conflicting data |
| `ExternalService` | 502 | CDN or external API failures |
| `Io` | 500 | Filesystem I/O errors |
| `Http` | 502 | Outbound HTTP request failures |
| `Internal` | 500 | Catch-all internal errors |

`ServiceError` implements both `IntoResponse` (for REST handlers) and conversion to async-graphql errors with structured extensions (for GraphQL resolvers).
