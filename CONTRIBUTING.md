# Contributing to Bevy Sphere Sweeper

Thank you for your interest in contributing! We welcome bug reports, feature requests, and code contributions to make this 3D Minesweeper experience even better.

## How to Contribute

### 1. Reporting Bugs
If you encounter an issue, please open a generic issue or report it with the following details:
*   Your operating system and graphics card.
*   The level you were on.
*   Steps to reproduce the bug.
*   Any error logs from the console.

### 2. Suggesting Features
Have an idea? We'd love to hear it! Please open an issue describing your idea and how it improves the gameplay or aesthetics.

### 3. Submitting Pull Requests (PRs)
1.  **Fork the repository** and create a new branch for your feature or fix.
    ```bash
    git checkout -b feature/your-feature-name
    ```
2.  **Make your changes.** Ensure your code follows the project's style (standard Rust formatting).
3.  **Test your changes.** Run the game and ensure nothing is broken.
    ```bash
    cargo run --release
    ```
4.  **Format your code** before committing:
    ```bash
    cargo fmt
    ```
5.  **Commit your changes** with clear, descriptive messages.
6.  **Push to your fork** and submit a Pull Request to the `main` branch.

## Development Guidelines

*   **Code Style:** We use `rustfmt` for code formatting. Please ensure your code is formatted before submitting.
*   **Bevy Version:** This project targets Bevy 0.16. Please ensure your changes are compatible.
*   **Performance:** Keep performance in mind, especially for the mesh generation and update loops, as the sphere grows significantly at higher levels.

## Code of Conduct

Please be respectful and kind to others. We are all here to learn and build cool things together.
