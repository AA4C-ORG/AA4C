---
name: aa4c
description: Use an installed AA4C engine through its MCP server or structured CLI to inspect nearby people and devices, send files or text, receive and fetch content, check transfers, diagnose connectivity, or begin a human-approved trust ceremony. Use when the user asks to “AA” or transfer something with AA4C, operate one of their AA4C devices, or query AA4C state. Do not use for downloading internet content, cloud storage, or chat; surface but never complete trust steps that require a person.
---

# AA4C

Drive AA4C through its command registry. Do not reproduce a hard-coded command catalog in this skill.

## Connect

1. Prefer the AA4C MCP server when available; its file roots and capability ceiling are the agent sandbox.
2. Otherwise run `aa4c commands --json` to discover the installed version's commands, schemas and capability tiers.
3. If neither is available, report that AA4C is not installed or running. Do not install or start a persistent service unless the user asked.

Use structured results. For CLI calls, add `--json` where supported or use:

```text
aa4c run <command-id> --params '<json>'
```

## Act safely

- Read state before acting: resolve the recipient with `devices.list`, `contacts.list` or `nearby.list`; never guess when names are ambiguous.
- Treat `read` as inspection, `act` as an external side effect, and `trust` as a relationship/security change. Never disguise one tier as another.
- Perform an `act` command when it is the user's requested outcome and the target and paths are unambiguous. Otherwise explain the ambiguity and ask.
- Do not complete human steps such as comparing a safety code, approving an unknown person, exporting recovery material, revoking a device or installing a plugin. Return the pending step to the user.
- Respect MCP read/write roots, allowed recipients, share permission and byte budgets. Do not use the CLI to bypass a denied path or target.
- For an in-person transfer, inspect the runtime proximity routes before suggesting Wi-Fi Direct, Bluetooth, infrared or a system handoff. Never claim a
  platform route exists from the device name alone, and never silently downgrade an AA4C-encrypted route to Web drop or system handoff.
- A system handoff requires a person to select and confirm the nearby device. Surface that step; do not interpret “submitted to the operating system” as
  “received and verified”. Remote receipt requires AA4C on both ends.
- Never read or reveal private keys, recovery codes, invitation secrets, full public keys or file contents unless the user explicitly asked for that content and the command permits it.

## Follow long work

When a command returns a transfer id, follow it with the event stream or `transfers.show`. Report completion, rejection or the structured failure reason; do not equate “command accepted” with “file delivered”. Use the error `hint` only as a next-step suggestion, not as authorization for a different action.

For `unreachable`, inspect `status.show` before retrying. Offer an explicit alternative such as waiting for the device or using a configured self-hosted relay. A one-time send link uses the same reachability and is not an offline cloud fallback.

## Keep runtime truth authoritative

The installed command registry outranks examples in documentation. If a command is absent or its schema differs, follow the registry and state the version mismatch. Prefer `config.get`/`config.set`; do not edit `config.toml` while the engine is running.
