use augment_chess_engine::{GameConfig, GameState};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let game = GameState::new(GameConfig::cardless(), 0)?;
    println!("{}", game.to_canonical_json()?);
    Ok(())
}
