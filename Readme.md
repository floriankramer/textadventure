# TextAdventure

This repo contians a tiny web-based engine for text adventure games. The core
design idea is to have the entire game driven by a single yaml file. If you
are looking for something more powerful consider RenPy or Quest.

## Building
The project uses [trunk](https://trunkrs.dev). Simply run `trunk serve` in the
root directory to run the game. The game data is read from a file called
`adventure.yaml` at the root directory. 

## The Adventure File
Look at `adventure.example.yaml` for an example adventure, or at adventure.rs
for the full file structure.


## Docker
TODO
