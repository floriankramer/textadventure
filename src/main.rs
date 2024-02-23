use audio::SongPlayer;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{AudioContext, HtmlElement, OscillatorType};

pub mod adventure;
pub mod audio;

fn main() {
    console_log::init().expect("unable to initalize the logging");
    console_error_panic_hook::set_once();

    let window = web_sys::window().expect("unable to get window");
    let document = window.document().expect("unable to get the document");

    let raw_adventure = include_str!("../adventure.yaml");
    let adventure: adventure::Document =
        serde_yaml::from_str(raw_adventure).expect("Expected a valid adventure");

    let callback = Closure::<dyn FnMut()>::new(move || {
        let player = SongPlayer::try_from(
            adventure
                .adventure
                .assets
                .music
                .get("hoist_the_colors")
                .expect("missing song")
                .clone(),
        )
        .expect("unable to parse the music");

        player.play();

        // let window = web_sys::window().expect("unable to get window");
        // let document = window.document().expect("unable to get the document");
        // document
        //     .get_element_by_id("play")
        //     .expect("Expected a play button")
        //     .dyn_ref::<HtmlElement>()
        //     .expect("The button should have been an html element")
        //     .set_inner_text("You clicked me");

        // let chord = [261.0, 329.0, 392.0];

        // let context = AudioContext::new().expect("unable to get an audio context");
        // for note in chord {
        //     let osc = context
        //         .create_oscillator()
        //         .expect("unable to create an oscillator");
        //     osc.set_type(OscillatorType::Square);
        //     osc.frequency().set_value(note);
        //     osc.connect_with_audio_node(&context.destination())
        //         .expect("Unable to connecto to audio node");
        //     osc.start_with_when(note as f64 / 261.0)
        //         .expect("unable to start the oscillator");
        // }
    });
    document
        .get_element_by_id("play")
        .expect("Expected a play button")
        .dyn_ref::<HtmlElement>()
        .expect("The button should have been an html element")
        .set_onclick(Some(callback.as_ref().unchecked_ref()));
    callback.forget();
}
