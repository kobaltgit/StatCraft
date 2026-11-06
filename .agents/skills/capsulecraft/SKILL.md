---
name: capsulecraft
description: Read, query, and update the CapsuleCraft project knowledge base and tasks for StatCraft.
---

# CapsuleCraft Knowledge Base Skill for StatCraft

This project is indexed into a local CapsuleCraft knowledge base located in `.capsule/`.
Every module, subsystem, architecture decision, task, and design document is stored
as an honest, self-contained RFC 5322 (`.eml`) capsule with full provenance.

## When to use this skill
- When exploring the architecture, public APIs, or subsystems of StatCraft.
- When checking technical debt, open tasks, and TODOs (`tasks_todo.eml`).
- When planning new features or cross-module refactorings.
- When saving architectural decisions or agent run artifacts.

## Available MCP Tools
If the CapsuleCraft MCP server is running (`capsulecraft`), use these tools:
- `project_overview`: Retrieve the structural overview of modules, subsystems, and metrics.
- `capsule_search`: Full-text BM25 search across capsules and code comments.
- `capsule_read`: Read a capsule by its Message-ID or file name.
- `capsule_create`: Create a new capsule documenting a feature, RFC, or bug analysis.
- `capsule_tasks`: List and filter codebase tasks and TODOs.

## Direct Capsule Access (Without MCP)
If MCP is not active, you can read the knowledge base directly from the filesystem:
- Root overview: `.capsule/index.eml`
- Subsystems: `.capsule/subsystem_*.eml`
- Module capsules: `.capsule/module_*.eml`
- Tasks: `.capsule/tasks_todo.eml`
