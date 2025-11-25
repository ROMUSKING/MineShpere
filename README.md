# Bevy Sphere Sweeper

A 3D Minesweeper game played on the surface of a geodesic sphere, built with the [Bevy Game Engine](https://bevyengine.org/) (v0.16).

## Features

*   **3D Gameplay:** Play Minesweeper on a fully rotatable sphere.
*   **Procedural Levels:** The sphere grows and becomes more complex as you level up.
    *   Level 1-2: 162 Cells (Subdivision 2)
    *   Level 3-5: 642 Cells (Subdivision 3)
    *   Level 6+: 2562 Cells (Subdivision 4)
*   **Intuitive Controls:**
    *   **Trackball Camera:** Smooth free-orbit camera navigation (no gimbal lock).
    *   **Invert Y:** Optional inverted vertical camera control.
    *   **Chording:** Double-click revealed cells to clear safe neighbors.
*   **Visual Feedback:**
    *   Color-coded tiles based on adjacent mine count.
    *   Distinct visual states for Hidden, Flagged, Revealed, Exploded, and Mines.

## Installation & Running

1.  **Prerequisites:** Ensure you have [Rust and Cargo](https://rustup.rs/) installed.
2.  **Clone the repository:**
    ```bash
    git clone https://github.com/ROMUSKING/MineShpere.git
    cd MineShpere
    ```
3.  **Run the game:**
    ```bash
    cargo run --release
    ```
    *Note: The `--release` flag is highly recommended for smooth performance, especially at higher levels.*

## Controls

| Action | Input | Description |
| :--- | :--- | :--- |
| **Reveal Cell** | `Left Click` | Reveals a hidden tile. Hitting a mine ends the game. |
| **Flag Cell** | `Right Click` | Marks a tile as a potential mine. Prevents accidental clicks. |
| **Chord** | `Double Left Click` | If a revealed tile has the correct number of flags around it, reveals all other neighbors. |
| **Orbit Camera** | `Right Mouse Drag` | Rotate the camera around the sphere. |
| **Zoom** | `Scroll Wheel` | Zoom in and out. |
| **Invert Y** | UI Button | Toggle vertical camera rotation direction (Top-Right corner). |

## Game Rules

1.  **Goal:** Reveal all "safe" cells on the sphere without detonating a mine.
2.  **Numbers:** A revealed number tells you how many mines are in the immediate adjacent cells (neighbors).
3.  **Winning:** The level is complete when all non-mine cells are revealed.
4.  **Losing:** Hitting a mine detonates it. You can restart the current level.
5.  **Progression:** Winning advances you to the next level, where the sphere gets larger and the mine density increases.

## Architecture

This project uses the Bevy ECS (Entity Component System).
*   **Entities:** Cells are individual entities with 3D meshes and materials.
*   **Systems:**
    *   `spawn_board`: Generates the Goldberg polyhedron geometry.
    *   `process_reveal_queue`: Handles the core game logic, flood-fill, and state updates.
    *   `camera_orbit_controls`: Implements the trackball camera logic.
*   **Plugins:** Uses `MeshPickingPlugin` for 3D interaction.

## License

This project is open-source.
