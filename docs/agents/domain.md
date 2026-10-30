# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

Layout: **single-context**.

## Before exploring, read these

- **`GLOSSARY.md`** at the repo root: the domain language. Code identifiers use its English names; UI text uses its Chinese names.
- **`docs/adr/`**: read ADRs that touch the area you're about to work in.
- **`book/src/`**: the design book. Find the chapter for the area via `book/src/SUMMARY.md`. Behaviour changes must update the matching chapter.

## File structure

```
/
├── AGENTS.md
├── GLOSSARY.md
├── docs/adr/
├── docs/agents/
└── book/src/
```

## Use the glossary's vocabulary

When your output names a domain concept (in an issue title, a refactor proposal, a hypothesis, a test name), use the term as defined in `GLOSSARY.md`. Don't drift to synonyms the glossary explicitly avoids.

If the concept you need isn't in the glossary yet, that's a signal: either you're inventing language the project doesn't use (reconsider) or there's a real gap (note it for `/domain-modeling`).

## Flag ADR conflicts

If your output contradicts an existing ADR, surface it explicitly rather than silently overriding:

> _Contradicts ADR-0007 (JSON control messages), but worth reopening because…_

Overturning an ADR means writing a new one; see `docs/adr/README.md`.
