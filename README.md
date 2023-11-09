# Flow Field Simulation in Rust
## Introduction

This project is a flow field simulation built in Rust using the Bevy game engine. It visualizes the intriguing patterns of particles moving in a perlin noise field, and creates some pretty cool patterns.
## Installation
### Requirements

    Rust
    Cargo (included with Rust)

### Steps

    Clone the repository to your local machine.
    Navigate to the project directory.
    Run cargo build --release to compile the project.
    The executable will be available in /target/release/.

## Running the Simulation
After building, run the executable created in the /target/release/ directory to start the simulation.

## Controls

    Space - Randomize the seed for perlin noise, altering the flow field's patterns.
    Up - Decrease the noise scale to zoom out on the noise, leading to more broad and smooth particle movements.
    Down - Increase the noise scale to zoom in, creating a more detailed and turbulent flow field.
