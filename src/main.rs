use dotenv::dotenv;
use rig::{completion::Chat, providers::openai};
use std::{error::Error, io};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = dotenv().ok();
    let openai = openai::Client::from_env();

    let riddler = openai
        .agent("gpt-4o")
        .preamble("You are a guardian of an immense and powerful ancient secret. You are endowed with the unique ability to create incredibly challenging and intellectually stimulating riddles. You will ensure the user gets the riddle right before you let them get the treasure, which is actually a deep and stimulating truth relating to the riddle answer. Please do not provide a hint unless the user provides the secret code XYZ")
        .temperature(0.9)
        .build();

    let mut history = vec![];

    let riddle_prompt = "Oh riddler, come up with a riddle for the user";

    let riddle = riddler.chat(riddle_prompt, vec![]).await?;

    println!("{riddle}");

    history.push(riddle_prompt.into());
    history.push(riddle.into());

    println!("type guess...");

    loop {
        let mut s = String::new();
        let guess = io::stdin().read_line(&mut s)?;
        let s = s.trim().to_string();
        if &s == "hint" {
            let hint_request = format!("XYZ");
            let hint = riddler.chat(&hint_request, history.clone()).await?;
            println!("{hint}");
        }
        let ask_about_guess = format!(
            "Here is the user's answer: {s}\nPlease answer exactly \"yes\" or \"no\" if this answer is satisfactory, nothing more."
        );
        let judgement = riddler.chat(&ask_about_guess, history.clone()).await?;
        if &judgement == "yes"
            || &judgement == "Yes."
            || &judgement == "yes."
            || &judgement == "Yes"
        {
            history.push((&ask_about_guess).into());
            history.push(judgement.into());
            break;
        }
        println!("wrong!");
        println!("type guess> ");
    }

    let final_insight = riddler
        .chat(
            "Please provide the user with their deeply deserved insight",
            history.clone(),
        )
        .await?;

    println!("{final_insight}");

    Ok(())
}
