# Bevy Sample App

This is a sample application using the Bevy game engine. It demonstrates basic game mechanics such as player movement, enemy AI, and camera control.

## Features

- 3D player character with capsule collider
- Enemy AI with vision and patrol behavior
- Camera follow and zoom functionality
- Game state management (playing, game over)

## Getting Started

To run this project, make sure you have [Rust](https://www.rust-lang.org/) installed. Then, clone the repository and run the following commands:

```bash
cargo build
cargo run
```

## Project Structure

```
src/
├── main.rs          # Entry point of the application
├── systems/         # Game systems (player input, enemy AI, etc.)
└── components/      # Game components (player, enemy, camera, etc.)
```

## Dependencies

This project uses the following dependencies:

- `bevy`: The main game engine
- `bevy_kira_audio`: Audio playback and control
- `bevy_rapier3d`: 3D physics and collision detection
