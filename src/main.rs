use chrono::Local;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use dotenv::dotenv;
use indicatif::{ProgressBar, ProgressStyle};
use rig::{
    completion::{Chat, Message},
    providers::openai,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs, path::Path, thread, time::Duration, io};

const SAVE_FILE: &str = "riddler_save.json";
const DIFFICULTY_DESCRIPTIONS: [&str; 3] = [
    "Easy: Simple riddles suitable for beginners",
    "Medium: Challenging riddles that will make you think",
    "Hard: Complex mind-benders for riddle masters",
];

const TITLE_ART: &str = r#"
 ____  _     _     _ _           
|  _ \(_) __| | __| | | ___ _ __ 
| |_) | |/ _` |/ _` | |/ _ \ '__|
|  _ <| | (_| | (_| | |  __/ |   
|_| \_\_|\__,_|\__,_|_|\___|_|   
                                 
"#;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct GameState {
    difficulty: usize,
    current_riddle: String,
    attempts: usize,
    hints_used: usize,
    history: Vec<Message>,
    score: i32,
    date_started: String,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            difficulty: 1,
            current_riddle: String::new(),
            attempts: 0,
            hints_used: 0,
            history: Vec::new(),
            score: 0,
            date_started: Local::now().to_rfc3339(),
        }
    }
}

fn show_spinner(message: &str, duration_ms: u64) {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    
    for _ in 0..duration_ms / 100 {
        pb.tick();
        thread::sleep(Duration::from_millis(100));
    }
    
    pb.finish_and_clear();
}

fn print_header() {
    let title = TITLE_ART.bright_cyan().bold();
    println!("\n{}", title);
    println!("{}", "The Ancient Guardian of Riddles".bright_purple().bold());
    println!("{}\n", "=".repeat(50).bright_blue());
}

fn print_fancy_message(message: &str, color: &str) {
    let formatted = match color {
        "red" => message.bright_red().bold(),
        "green" => message.bright_green().bold(),
        "blue" => message.bright_blue().bold(),
        "yellow" => message.bright_yellow().bold(),
        "cyan" => message.bright_cyan().bold(),
        "magenta" => message.bright_magenta().bold(),
        "white" => message.bright_white().bold(),
        _ => message.normal(),
    };
    
    println!("\n{}", formatted);
}

fn save_game(state: &GameState) -> Result<(), Box<dyn Error>> {
    // Create a temporary file to write to first
    let temp_file = format!("{}.tmp", SAVE_FILE);
    let json = serde_json::to_string_pretty(state)?;
    
    // Write to the temporary file first
    fs::write(&temp_file, &json)?;
    
    // Then rename the temporary file to the actual save file
    // This helps prevent corruption if the program crashes during the write
    if Path::new(&temp_file).exists() {
        if Path::new(SAVE_FILE).exists() {
            fs::remove_file(SAVE_FILE)?;
        }
        fs::rename(&temp_file, SAVE_FILE)?;
    }
    
    Ok(())
}

fn load_game() -> Result<GameState, Box<dyn Error>> {
    if Path::new(SAVE_FILE).exists() {
        let json = fs::read_to_string(SAVE_FILE)?;
        let state: GameState = serde_json::from_str(&json)?;
        Ok(state)
    } else {
        Ok(GameState::default())
    }
}

fn get_difficulty_prompt(difficulty: usize) -> &'static str {
    match difficulty {
        0 => "Please create a simple and straightforward riddle suitable for beginners.",
        1 => "Create a moderately challenging riddle that requires some thought.",
        2 => "Craft an extremely challenging riddle that will truly test the user's intellect.",
        _ => "Come up with a riddle for the user",
    }
}

fn calculate_score(difficulty: usize, attempts: usize, hints: usize) -> i32 {
    let base_score = match difficulty {
        0 => 10,
        1 => 25,
        2 => 50,
        _ => 25,
    };
    
    let attempt_penalty = (attempts as i32 - 1).max(0) * 5;
    let hint_penalty = hints as i32 * 10;
    
    (base_score - attempt_penalty - hint_penalty).max(0)
}

/// Helper function to handle API calls with consistent error handling
async fn guardian_chat<C>(
    riddler: &C,
    prompt: &str,
    history: Vec<Message>,
    error_message: &str,
    spinner_message: &str,
    spinner_duration: u64,
) -> Result<String, Box<dyn Error>>
where
    C: Chat,
{
    show_spinner(spinner_message, spinner_duration);
    
    match riddler.chat(prompt, history).await {
        Ok(response) => Ok(response),
        Err(e) => {
            print_fancy_message("The Ancient Guardian cannot respond...", "red");
            println!("Error: {}", e);
            Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                error_message,
            )))
        }
    }
}

async fn start_new_game(
    riddler: &impl Chat,
    difficulty: usize,
) -> Result<GameState, Box<dyn Error>> {
    let mut state = GameState {
        difficulty,
        date_started: Local::now().to_rfc3339(),
        ..Default::default()
    };
    
    // Create the riddle prompt based on difficulty
    let riddle_prompt = get_difficulty_prompt(difficulty);
    
    let riddle = guardian_chat(
        riddler,
        riddle_prompt,
        vec![],
        "Failed to communicate with the Guardian",
        "The Ancient Guardian is thinking of a riddle...",
        3000,
    )
    .await?;
    
    state.current_riddle = riddle.clone();
    
    // Add to history
    state.history.push(Message::user(riddle_prompt));
    state.history.push(Message::assistant(&riddle));
    
    save_game(&state)?;
    
    Ok(state)
}

async fn get_hint(
    riddler: &impl Chat,
    state: &mut GameState,
) -> Result<String, Box<dyn Error>> {
    let hint_request = "XYZ".to_string();
    state.hints_used += 1;
    
    let hint = guardian_chat(
        riddler,
        &hint_request,
        state.history.clone(),
        "Failed to get a hint from the Guardian",
        "The Guardian is considering a hint...",
        2000,
    )
    .await?;
    
    state.history.push(Message::user(&hint_request));
    state.history.push(Message::assistant(&hint));
    
    save_game(state)?;
    
    Ok(hint)
}

async fn check_guess(
    riddler: &impl Chat,
    guess: &str,
    state: &mut GameState,
) -> Result<bool, Box<dyn Error>> {
    state.attempts += 1;
    
    let ask_about_guess = format!(
        "Here is the user's answer: {}\nPlease answer exactly \"yes\" or \"no\" if this answer is satisfactory, nothing more.",
        guess
    );
    
    let judgement = guardian_chat(
        riddler,
        &ask_about_guess,
        state.history.clone(),
        "Failed to get judgment from the Guardian",
        "The Guardian is judging your answer...",
        1500,
    )
    .await?;
    
    // Trim and convert to lowercase for more reliable comparison
    let judgement_clean = judgement.trim().to_lowercase();
    let correct = judgement_clean == "yes" || judgement_clean == "yes.";
    
    state.history.push(Message::user(&ask_about_guess));
    state.history.push(Message::assistant(&judgement));
    
    if correct {
        state.score += calculate_score(state.difficulty, state.attempts, state.hints_used);
    }
    
    save_game(state)?;
    
    Ok(correct)
}

async fn reveal_insight(
    riddler: &impl Chat,
    state: &mut GameState,
) -> Result<String, Box<dyn Error>> {
    let insight_prompt = "Please provide the user with their deeply deserved insight";
    
    let insight = guardian_chat(
        riddler,
        insight_prompt,
        state.history.clone(),
        "Failed to get insight from the Guardian",
        "The Guardian is preparing your insight...",
        3000,
    )
    .await?;
    
    state.history.push(Message::user(insight_prompt));
    state.history.push(Message::assistant(&insight));
    
    save_game(state)?;
    
    Ok(insight)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = dotenv().ok();
    let openai = openai::Client::from_env();
    
    let riddler = openai
        .agent("gpt-4o")
        .preamble("You are a guardian of an immense and powerful ancient secret. You are endowed with the unique ability to create incredibly challenging and intellectually stimulating riddles. You will ensure the user gets the riddle right before you let them get the treasure, which is actually a deep and stimulating truth relating to the riddle answer. Please do not provide a hint unless the user provides the secret code XYZ. Your responses should be mystical, ancient, and fitting for a wise guardian of secrets. For hints, be enigmatic but helpful.")
        .temperature(0.9)
        .build();
    
    // Main game loop
    loop {
        print_header();
        
        let selections = vec!["Start New Game", "Continue Saved Game", "View Instructions", "Quit"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose an option:")
            .default(0)
            .items(&selections)
            .interact()?;
        
        match selection {
            0 => {
                // Start New Game
                print_fancy_message("Choose difficulty level:", "cyan");
                let difficulty = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Difficulty")
                    .default(1)
                    .items(&DIFFICULTY_DESCRIPTIONS)
                    .interact()?;
                
                let mut state = start_new_game(&riddler, difficulty).await?;
                
                print_fancy_message("The Ancient Guardian speaks:", "yellow");
                println!("{}", state.current_riddle.bright_white());
                
                // Riddle solving loop
                loop {
                    println!("\n{}", "-".repeat(50).bright_blue());
                    println!("Attempts: {} | Hints Used: {} | Score: {}", 
                             state.attempts.to_string().yellow(),
                             state.hints_used.to_string().yellow(),
                             state.score.to_string().green());
                    println!("{}", "-".repeat(50).bright_blue());
                    
                    let guess: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Your answer (type 'hint' for a hint, 'riddle' to see the riddle again)")
                        .interact_text()?;
                    
                    if guess.trim().to_lowercase() == "hint" {
                        let hint = get_hint(&riddler, &mut state).await?;
                        print_fancy_message("The Guardian whispers a hint:", "magenta");
                        println!("{}", hint.bright_white());
                        continue;
                    }
                    
                    if guess.trim().to_lowercase() == "riddle" {
                        print_fancy_message("The Guardian repeats the riddle:", "yellow");
                        println!("{}", state.current_riddle.bright_white());
                        continue;
                    }
                    
                    // Check the guess
                    let correct = check_guess(&riddler, &guess, &mut state).await?;
                    
                    if correct {
                        print_fancy_message("CORRECT!", "green");
                        println!("{}", "The Ancient Guardian nods in approval...".bright_green());
                        
                        // Get the final insight
                        let insight = reveal_insight(&riddler, &mut state).await?;
                        
                        print_fancy_message("The Guardian reveals the promised wisdom:", "cyan");
                        println!("{}", insight.bright_white());
                        
                        println!("\n{} {}", "Final Score:".bright_yellow(), state.score.to_string().bright_green());
                        
                        println!("\nWould you like to play another riddle?");
                        let play_again = Select::with_theme(&ColorfulTheme::default())
                            .with_prompt("Choose an option")
                            .default(0)
                            .items(&["Yes", "No"])
                            .interact()?;
                        
                        if play_again == 0 {
                            break; // Break out of riddle loop to start a new game
                        } else {
                            return Ok(());
                        }
                    } else {
                        print_fancy_message("INCORRECT!", "red");
                        println!("{}", "The Ancient Guardian shakes their head. Try again...".bright_red());
                    }
                }
            }
            1 => {
                // Continue Saved Game
                match load_game() {
                    Ok(mut state) => {
                        if state.current_riddle.is_empty() {
                            print_fancy_message("No saved game found!", "red");
                            thread::sleep(Duration::from_secs(2));
                            continue;
                        }
                        
                        print_fancy_message("Continuing your quest...", "blue");
                        println!("Difficulty: {} | Attempts: {} | Hints: {}", 
                                 DIFFICULTY_DESCRIPTIONS[state.difficulty].yellow(),
                                 state.attempts.to_string().yellow(),
                                 state.hints_used.to_string().yellow());
                        
                        print_fancy_message("The Ancient Guardian's riddle:", "yellow");
                        println!("{}", state.current_riddle.bright_white());
                        
                        // Continue riddle solving loop
                        loop {
                            println!("\n{}", "-".repeat(50).bright_blue());
                            println!("Attempts: {} | Hints Used: {} | Score: {}", 
                                     state.attempts.to_string().yellow(),
                                     state.hints_used.to_string().yellow(),
                                     state.score.to_string().green());
                            println!("{}", "-".repeat(50).bright_blue());
                            
                            let guess: String = Input::with_theme(&ColorfulTheme::default())
                                .with_prompt("Your answer (type 'hint' for a hint, 'riddle' to see the riddle again)")
                                .interact_text()?;
                            
                            if guess.trim().to_lowercase() == "hint" {
                                let hint = get_hint(&riddler, &mut state).await?;
                                print_fancy_message("The Guardian whispers a hint:", "magenta");
                                println!("{}", hint.bright_white());
                                continue;
                            }
                            
                            if guess.trim().to_lowercase() == "riddle" {
                                print_fancy_message("The Guardian repeats the riddle:", "yellow");
                                println!("{}", state.current_riddle.bright_white());
                                continue;
                            }
                            
                            // Check the guess
                            let correct = check_guess(&riddler, &guess, &mut state).await?;
                            
                            if correct {
                                print_fancy_message("CORRECT!", "green");
                                println!("{}", "The Ancient Guardian nods in approval...".bright_green());
                                
                                // Get the final insight
                                let insight = reveal_insight(&riddler, &mut state).await?;
                                
                                print_fancy_message("The Guardian reveals the promised wisdom:", "cyan");
                                println!("{}", insight.bright_white());
                                
                                println!("\n{} {}", "Final Score:".bright_yellow(), state.score.to_string().bright_green());
                                
                                println!("\nWould you like to play another riddle?");
                                let play_again = Select::with_theme(&ColorfulTheme::default())
                                    .with_prompt("Choose an option")
                                    .default(0)
                                    .items(&["Yes", "No"])
                                    .interact()?;
                                
                                if play_again == 0 {
                                    break; // Break out of riddle loop to start a new game
                                } else {
                                    return Ok(());
                                }
                            } else {
                                print_fancy_message("INCORRECT!", "red");
                                println!("{}", "The Ancient Guardian shakes their head. Try again...".bright_red());
                            }
                        }
                    }
                    Err(_) => {
                        print_fancy_message("No saved game found or error loading save!", "red");
                        thread::sleep(Duration::from_secs(2));
                    }
                }
            }
            2 => {
                // View Instructions
                print_fancy_message("HOW TO PLAY", "blue");
                println!("{}", "Welcome, seeker of ancient wisdom!".bright_cyan());
                println!("{}", "In this game, you will face the Ancient Guardian who will test your wit with riddles.".bright_white());
                println!("{}", "Solve the riddle correctly to receive a profound insight.".bright_white());
                println!("\n{}", "Game Features:".bright_yellow());
                println!("• Three difficulty levels");
                println!("• Hint system (type 'hint' when stuck)");
                println!("• Scoring based on difficulty, attempts, and hints used");
                println!("• Automatic game saving");
                
                println!("\n{}", "Commands during play:".bright_yellow());
                println!("• Type 'hint' to request a hint (reduces score)");
                println!("• Type 'riddle' to see the riddle again");
                
                println!("\n{}", "Press Enter to return to the main menu...".bright_cyan());
                let _: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("")
                    .allow_empty(true)
                    .interact_text()?;
            }
            3 => {
                // Quit
                print_fancy_message("Farewell, seeker of wisdom!", "cyan");
                thread::sleep(Duration::from_secs(1));
                break;
            }
            _ => unreachable!(),
        }
    }
    
    Ok(())
}
