use game::Game;

pub mod adventure;
pub mod audio;
pub mod game;

fn main() {
    console_log::init().expect("unable to initalize the logging");
    console_error_panic_hook::set_once();

    // Load the adventure
    let raw_adventure = include_str!("../adventure.yaml");

    // Parse the yaml
    let adventure: adventure::Document = match serde_yaml::from_str(raw_adventure) {
        Ok(a) => a,
        Err(err) => {
            log::error!("Unable to parse the adventure: {err:#}");
            return;
        }
    };

    let mut game = match Game::try_from(adventure.adventure) {
        Ok(g) => g,
        Err(err) => {
            log::error!("Unable to parse the adventure: {err:#}");
            return;
        }
    };

    if let Err(err) = game.start() {
      log::error!("Unable to start the game: {err:#}");
    }
}
