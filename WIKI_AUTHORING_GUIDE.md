# Hidow Wiki Authoring Guide

> **Audience**: LLM agents, developers, or documentation tools that need to convert project documentation into a format that `hidow` can ingest into its Knowledge Graph.

---

## Quick Start

```
wiki/                          ← Root folder (pass to hidow via --wiki-path)
├── overview.md                ← Root-level pages (type = filename)
├── modules/                   ← Folder name = plural of type
│   ├── auth.md
│   └── billing.md
├── entities/
│   ├── user.md
│   └── invoice.md
├── concepts/
│   └── multi-tenancy.md
├── flows/
│   └── checkout-flow.md
└── questions/
    └── why-soft-delete.md
```

```bash
# Ingest into hidow
hidow -i my_project init
hidow -i my_project --wiki-path ./wiki ingest --full
```

---

## Page Structure

Every `.md` file has two parts:

```markdown
---
<YAML frontmatter>   ← Metadata (structured, required)
---

<Markdown body>      ← Content (free-form, used for embeddings & RAG)
```

### Frontmatter Schema

```yaml
# ─── REQUIRED ───────────────────────────────────
title: "Human-readable page title"
type: module                    # Node type (see "Page Types" below)

# ─── OPTIONAL ───────────────────────────────────
status: current                 # Any string: current, draft, deprecated, active...
tags:                           # Flat list of keywords (used in search)
  - billing
  - payment
sources:                        # Source documents this page was derived from
  - "SRS_Billing_v2.1.md"
  - "Meeting notes 2026-01-15"

# ─── RELATIONSHIPS (edges in the graph) ─────────
relationships:
  - target: wiki/modules/auth   # Path to another wiki page (without .md)
    type: depends_on            # Edge type (see "Edge Types" below)
    label: "Auth check before payment"  # Human-readable description

# ─── BUSINESS RULES (module pages only) ─────────
business_rules:
  - id: BR_BIL_001              # Unique ID across the project
    rule: "Invoice cannot be deleted after payment"
    severity: critical          # critical | warning | info
    affects:                    # Entities affected by this rule
      - wiki/entities/invoice

# ─── ATTRIBUTES (entity pages only) ─────────────
attributes:
  - name: invoice_no
    type: auto                  # auto | string | number | date | enum | boolean | reference
    required: true
    description: "Auto-generated invoice number"
  - name: status
    type: enum
    required: true
    enum_values: ["draft", "sent", "paid", "cancelled"]
    description: "Invoice lifecycle status"
  - name: customer_id
    type: reference
    reference: wiki/entities/customer
    description: "Link to customer"

# ─── DATA FLOW (flow pages only) ────────────────
data_flow:
  - step: 1
    name: "Create cart"
    module: wiki/modules/cart
    input: ["Product selection"]
    output: ["wiki/entities/cart"]
  - step: 2
    name: "Process payment"
    module: wiki/modules/billing
    input: ["wiki/entities/cart"]
    output: ["wiki/entities/invoice", "wiki/entities/payment"]
```

---

## Page Types

Hidow supports **any** page type. Use `type: <name>` in frontmatter, and the folder name should be the **plural** form.

### Recommended Types

| Type | Folder | Purpose | Special Fields |
|------|--------|---------|----------------|
| `module` | `modules/` | Functional components, services, subsystems | `business_rules` |
| `entity` | `entities/` | Data models, database tables, domain objects | `attributes` |
| `concept` | `concepts/` | Domain concepts, terminology, patterns | — |
| `flow` | `flows/` | Workflows, processes, data pipelines | `data_flow` |
| `question` | `questions/` | FAQs, design decisions, trade-offs | — |
| `overview` | root level | System architecture, high-level documentation | — |

### Custom Types

You can create **any custom type**. Hidow auto-creates the schema on ingest:

```
wiki/
├── integrations/           ← type: integration
│   └── stripe-api.md
├── policies/               ← type: policy
│   └── data-retention.md
└── runbooks/               ← type: runbook
    └── deploy-production.md
```

```yaml
# wiki/integrations/stripe-api.md
---
title: "Stripe Payment Integration"
type: integration
tags: [stripe, payment, api]
relationships:
  - target: wiki/modules/billing
    type: uses
    label: "Payment processing"
---
```

> **How it works**: Folder names are automatically singularized:
> `modules→module`, `entities→entity`, `policies→policy`, `integrations→integration`, etc.
>
> The singularization handles common English plural rules including `-ies`, `-es`, and `-s` suffixes.

---

## Edge Types

Edges represent relationships between nodes. Use these in the `relationships[].type` field:

| Edge Type | Meaning | Example |
|-----------|---------|---------|
| `depends_on` | A requires B to function | Module A depends on Module B |
| `produces` | A creates/outputs B | Module creates Entity |
| `consumes` | A reads/uses data from B | Module reads Entity |
| `contains` | A structurally contains B | Entity has sub-Entity |
| `part_of` | A is a component of B | Entity belongs to Entity |
| `implements` | A implements concept B | Module implements Concept |
| `uses` | General usage relationship | Any node uses any node |
| `triggers` | A causes B to execute | Module triggers Module |
| `affects` | Business rule constrains entity | (auto-created from `business_rules[].affects`) |

> **Tip**: When unsure, use `uses`. It's the most generic edge type.

---

## Path Conventions

All `target` fields in `relationships`, `affects`, `data_flow.module`, `data_flow.input/output`, and `attributes[].reference` use **wiki-relative paths without `.md`**:

```
wiki/{folder}/{filename-without-extension}
```

### Examples

| File on disk | Path in YAML |
|-------------|-------------|
| `wiki/modules/billing.md` | `wiki/modules/billing` |
| `wiki/entities/invoice.md` | `wiki/entities/invoice` |
| `wiki/flows/checkout-flow.md` | `wiki/flows/checkout-flow` |
| `wiki/overview.md` | `wiki/overview` |

### Slug Rules

The filename (without `.md`) becomes the **slug** (record ID in the database):
- Hyphens → underscores: `checkout-flow.md` → `checkout_flow`
- Used as: `flow:checkout_flow`

---

## Skipped Files

Hidow automatically skips these files during parsing:
- `index.md` — Use for navigation/TOC only
- `log.md` — Use for changelog/notes

---

## Markdown Body Guidelines

The markdown body (after `---`) is stored as `content` and used for:
1. **Semantic search** — Embedded as a vector (first ~500 chars + title + tags)
2. **RAG retrieval** — Full content returned by `hidow query ask`
3. **Content display** — Shown by `hidow query content <id>`

### Recommended Structure

```markdown
# Page Title (repeat from frontmatter)

## Overview / Definition
Brief description of what this is.

## Details
Detailed documentation, business logic, implementation notes.

## Examples (optional)
Code samples, screenshots, API examples.

## Notes / Open Questions (optional)
Things to clarify or revisit.
```

### Tips for Better Search Quality
- **Start with a clear definition** — The first ~500 characters are weighted heavily in embeddings
- **Use domain keywords** in the body — Not just in tags
- **Be specific** — "Invoice must have at least one line item" > "Must validate"
- **Include both languages** if bilingual — Hidow's embedding model supports multilingual text

---

## Complete Example: E-Commerce Project

### `wiki/overview.md`

```markdown
---
title: "E-Commerce Platform Overview"
type: overview
tags: [architecture, e-commerce]
status: current
---

# E-Commerce Platform Overview

Modern e-commerce platform built with microservices architecture...
```

### `wiki/modules/billing.md`

```markdown
---
title: "Billing Module"
type: module
status: current
tags: [billing, payment, invoice]
sources:
  - "SRS_Billing_v2.1.md"
relationships:
  - target: wiki/modules/auth
    type: depends_on
    label: "User authentication before payment"
  - target: wiki/modules/notification
    type: triggers
    label: "Send invoice email after payment"
  - target: wiki/entities/invoice
    type: produces
    label: "Creates invoices"
  - target: wiki/entities/payment
    type: produces
    label: "Records payments"
business_rules:
  - id: BR_BIL_001
    rule: "Cannot delete invoice after payment is recorded"
    severity: critical
    affects:
      - wiki/entities/invoice
  - id: BR_BIL_002
    rule: "Refund amount cannot exceed original payment"
    severity: critical
    affects:
      - wiki/entities/payment
  - id: BR_BIL_003
    rule: "All amounts stored in cents (integer) to avoid float precision"
    severity: warning
---

# Billing Module

## Overview
Handles all financial transactions including invoice generation,
payment processing, and refund management.

## Key Features
- Invoice generation from cart
- Multiple payment methods (Stripe, bank transfer)
- Automatic tax calculation
- Refund processing with approval workflow
```

### `wiki/entities/invoice.md`

```markdown
---
title: "Invoice"
type: entity
status: current
tags: [entity, invoice, billing, financial]
relationships:
  - target: wiki/entities/customer
    type: uses
    label: "Invoice belongs to customer"
  - target: wiki/modules/billing
    type: uses
    label: "Managed by billing module"
attributes:
  - name: invoice_no
    type: auto
    required: true
    description: "Auto-generated invoice number (INV-YYYYMMDD-XXXX)"
  - name: status
    type: enum
    required: true
    enum_values: ["draft", "sent", "paid", "overdue", "cancelled"]
    description: "Invoice lifecycle status"
  - name: total_cents
    type: number
    required: true
    description: "Total amount in cents"
  - name: currency
    type: string
    required: true
    description: "ISO 4217 currency code"
  - name: customer_id
    type: reference
    required: true
    reference: wiki/entities/customer
    description: "Customer this invoice belongs to"
  - name: due_date
    type: date
    required: false
    description: "Payment due date"
---

# Invoice

## Definition
An invoice represents a financial document issued to a customer...
```

### `wiki/flows/checkout-flow.md`

```markdown
---
title: "Checkout Flow"
type: flow
status: current
tags: [checkout, payment, workflow]
relationships:
  - target: wiki/modules/cart
    type: uses
    label: "Cart management"
  - target: wiki/modules/billing
    type: uses
    label: "Payment processing"
data_flow:
  - step: 1
    name: "Validate cart"
    module: wiki/modules/cart
    input: ["Cart items", "Stock availability"]
    output: ["Validated cart"]
  - step: 2
    name: "Calculate totals"
    module: wiki/modules/billing
    input: ["Validated cart", "Tax rules"]
    output: ["wiki/entities/invoice"]
  - step: 3
    name: "Process payment"
    module: wiki/modules/billing
    input: ["wiki/entities/invoice", "Payment method"]
    output: ["wiki/entities/payment"]
  - step: 4
    name: "Send confirmation"
    module: wiki/modules/notification
    input: ["wiki/entities/invoice", "wiki/entities/payment"]
    output: ["Email notification"]
---

# Checkout Flow

## Overview
End-to-end flow from cart validation to payment confirmation...
```

---

## Validation Checklist

Before running `hidow ingest`, verify:

- [ ] Every `.md` file has `---` delimited YAML frontmatter
- [ ] Every page has `title` and `type` fields
- [ ] All `target` paths use format `wiki/{folder}/{slug}` (no `.md`)
- [ ] All `target` paths point to files that actually exist
- [ ] Business rule IDs are unique across the entire wiki
- [ ] Folder names match the plural of their page type
- [ ] No circular `depends_on` chains (optional, but recommended)
- [ ] Run `hidow -i <name> ingest --dry-run` to preview before writing

```bash
# Quick validation
hidow -i my_project --wiki-path ./wiki ingest --dry-run

# Full ingest
hidow -i my_project --wiki-path ./wiki ingest --full

# Check graph health
hidow -i my_project --wiki-path ./wiki lint
```

---

## Prompt for LLM Wiki Generation

Use this prompt to instruct an LLM to convert existing documentation into hidow-compatible wiki pages:

```
You are converting project documentation into a structured wiki for the `hidow` Knowledge Graph CLI.

For each piece of documentation, create a markdown file with YAML frontmatter following this schema:

Required fields:
- title: Human-readable title
- type: One of: module, entity, concept, flow, question, overview (or custom)

Optional fields:
- status: current | draft | deprecated
- tags: list of keywords
- sources: list of source documents
- relationships: list of {target, type, label} where target is "wiki/{folder}/{slug}"
- business_rules: (modules only) list of {id, rule, severity, affects}
- attributes: (entities only) list of {name, type, required, description}
- data_flow: (flows only) list of {step, name, module, input, output}

Edge types for relationships: depends_on, produces, consumes, contains, part_of, implements, uses, triggers

File naming: Use kebab-case for filenames (e.g., user-management.md).
Path format: wiki/{plural-folder}/{filename-without-md} (e.g., wiki/modules/user-management)

After frontmatter, write the markdown body with detailed documentation content.
```
