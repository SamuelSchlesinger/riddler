# Riddler: The Ancient Guardian of Riddles

```
 ____  _     _     _ _           
|  _ \(_) __| | __| | | ___ _ __ 
| |_) | |/ _` |/ _` | |/ _ \ '__|
|  _ <| | (_| | (_| | |  __/ |   
|_| \_\_|\__,_|\__,_|_|\___|_|   
```

## About

Riddler is an interactive command-line game where you face the Ancient Guardian of Riddles - a mystical entity powered by GPT-4o that will challenge your wit and reward your intellect. Solve riddles of varying difficulty to unlock profound insights and wisdom.

## Features

- **Beautiful CLI Interface**: Enjoy a visually appealing terminal experience with colorful text, progress spinners, and ASCII art
- **Three Difficulty Levels**: Test your skills with easy, medium, or hard riddles
- **Dynamic AI-Generated Content**: Every riddle is uniquely crafted by the Guardian (GPT-4o)
- **Hint System**: Stuck on a riddle? Ask for a hint (but beware the score penalty!)
- **Scoring System**: Earn points based on difficulty, attempts, and hints used
- **Auto-Save**: Your game progress is automatically saved so you can continue your quest later
- **Profound Rewards**: Every correct answer unlocks a piece of wisdom related to the riddle

## Installation

1. Ensure you have Rust installed ([install Rust](https://www.rust-lang.org/tools/install))
2. Clone this repository
3. Create a `.env` file with your OpenAI API key:
   ```
   OPENAI_API_KEY=your_api_key_here
   ```
4. Build and run the game:
   ```
   cargo build --release
   cargo run --release
   ```

## How to Play

1. Start a new game or continue a saved one
2. Choose your difficulty level: Easy, Medium, or Hard
3. The Ancient Guardian will present you with a riddle
4. Type your answer and press Enter
5. If correct, you'll receive wisdom and points
6. If incorrect, keep trying!

### Special Commands

- Type `hint` to receive a hint from the Guardian
- Type `riddle` to see the current riddle again

## Requirements

- Rust 2024 Edition
- OpenAI API key (for GPT-4o access)

## License

This project is open source and available under the MIT License.

---

*"Wisdom awaits those who unravel the Guardian's mysteries..."*