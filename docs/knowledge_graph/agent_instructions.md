# Coding Agent Instructions: Knowledge Graph Maintenance

## Overview
This document outlines the procedures for populating and maintaining the `knowledge_graph.json` file for the `MineSphere` project. The knowledge graph provides a structured representation of the codebase architecture, dependencies, and components.

## Schema Compliance
All entries in the knowledge graph MUST adhere to the JSON schema defined in `docs/knowledge_graph/schema.json`.

### Entities
- **id**: A unique identifier for the entity (e.g., `module:main`, `struct:GameSession`).
- **type**: One of the allowed types (`Module`, `Service`, `Component`, `Struct`, `System`, etc.).
- **name**: The human-readable name of the entity.
- **description**: A brief summary of the entity's purpose.
- **attributes**: Additional metadata like `path`, `language` (Rust), `visibility`.

### Relationships
- **sourceId**: The ID of the source entity.
- **targetId**: The ID of the target entity.
- **type**: The nature of the relationship (`depends_on`, `uses`, `contains`, `observes`, `reads`, `writes`).

## Population Strategy

### Automated Extraction (Future Implementation)
- Use `syn` (Rust AST parser) to scan `src/**/*.rs` files.
- Extract:
    - **Structs/Enums** as `Component` or `Struct` entities.
    - **Functions** annotated with `#[system]` or added to `App` as `System` entities.
    - **Resources** and **Events** derived from `Resource` or `Event` traits.
- Infer relationships:
    - **contains**: File -> Struct/Function.
    - **uses**: System -> Component/Resource (via `Query`, `Res`, `ResMut`).
    - **observes**: System -> Event (via `EventReader`, `Trigger`).
    - **writes**: System -> Event (via `EventWriter`).

### Manual Curation
- High-level architecture components (e.g., "Game Loop", "Rendering Subsystem") should be manually added to group lower-level entities.
- Logical features (e.g., "Minesweeper Logic", "Camera Control") should be defined as `Feature` entities.

## Maintenance Workflow
1.  **Pre-Commit**: When modifying architecture or adding new systems/components, update `knowledge_graph.json` to reflect changes.
2.  **Validation**: Ensure the JSON file validates against `schema.json`.
3.  **Review**: Pull requests should include diffs to the knowledge graph if structural changes occurred.

## Initial Population (Current State)
The graph should initially be populated with the core entities found in `src/main.rs`:
- **App**: The main Bevy app entry point.
- **Systems**: `setup_scene`, `setup_ui`, `generate_board`, `on_cell_click`, etc.
- **Components**: `Cell`, `CellVisuals`, `HudText`.
- **Resources**: `GameSession`.
- **Events**: `RevealCell`, `ChordCell`.
