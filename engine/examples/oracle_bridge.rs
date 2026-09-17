use augment_chess_engine::*;
use serde::Deserialize;
use serde_json::json;
use std::io::{self, BufRead};
#[derive(Deserialize)]
struct Request {
    state: Option<CanonicalState>,
    action: Option<Action>,
}
fn main() {
    for line in io::stdin().lock().lines() {
        let output = (|| -> Result<_, Box<dyn std::error::Error>> {
            let request: Request = serde_json::from_str(&line?)?;
            let mut game = match request.state {
                Some(s) => GameState::from_snapshot(s)?,
                None => GameState::new(GameConfig::cardless(), 0)?,
            };
            if let Some(a) = request.action {
                game.apply_action(a)?;
            }
            Ok(
                json!({"state":game.snapshot(),"actions":game.legal_actions()?,"check":{"white":game.is_in_check(Color::White)?,"black":game.is_in_check(Color::Black)?}}),
            )
        })();
        match output {
            Ok(v) => println!("{v}"),
            Err(e) => println!("{}", json!({"error":e.to_string()})),
        }
    }
}
