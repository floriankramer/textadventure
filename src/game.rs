use std::{
  collections::{HashMap, HashSet},
  rc::Rc,
  sync::{Arc, Mutex},
};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::HtmlElement;

use crate::{
  adventure::{Action, Adventure, Room},
  audio::SongPlayer,
};

pub struct Game {
  data: Rc<Mutex<GameData>>,
}

struct GameData {
  intro: String,
  start: String,
  rooms: HashMap<String, Room>,
  music: HashMap<String, SongPlayer>,

  current_text: String,
  current_room: String,

  inventory: HashSet<String>,

  text_element: HtmlElement,
  actions_element: HtmlElement,
}

impl Game {
  pub fn start(&mut self) -> Result<()> {
    let mut data = self.data.lock().unwrap();

    data.current_text = data.intro.clone();
    data.text_element.set_inner_html(&data.current_text);

    let actions = vec![Action {
      name: "Start".to_string(),
      transition: Some(data.start.clone()),
      ..Default::default()
    }];

    Self::update_actions(&mut data, self.data.clone(), &actions)?;

    Self::load(&mut data, self.data.clone());

    Ok(())
  }

  fn goto_room(data: &mut GameData, data_ptr: Rc<Mutex<GameData>>, room: &str) {
    data.current_room = room.to_string();

    let room = match data.rooms.get(room) {
      Some(r) => r.clone(),
      None => {
        log::error!("Tried to go to nonexistant room {room}");
        return;
      }
    };

    data.current_text += &room.description;
    data.text_element.set_inner_html(&data.current_text);

    // Item states might have changed
    if let Err(err) = Self::update_actions(data, data_ptr.clone(), &room.actions) {
      log::error!("Unable to update the actions: {err:#}");
    }
  }

  fn update_actions(
    data: &mut GameData,
    data_ptr: Rc<Mutex<GameData>>,
    actions: &[Action],
  ) -> Result<()> {
    data.actions_element.set_inner_html("");

    let document = web_sys::window().unwrap().document().unwrap();

    for action in actions {
      // Check if the action's requirements are met
      let skip = (|| {
        for item in &action.depends.on {
          log::info!("Checking if we have {item}");
          if !data.inventory.contains(item) {
            return true;
          }
        }

        for item in &action.depends.not {
          log::info!("Checking if we don't have {item}");
          if data.inventory.contains(item) {
            return true;
          }
        }
        false
      })();
      if skip {
        continue;
      }

      // Create a new link element
      let link: HtmlElement = document
        .create_element("a")
        .map_err(js_to_anyhow)?
        .dyn_into()
        .map_err(|_| anyhow!("Expected a link"))?;

      // Set the text, and add a pseudo target to make the link clickable
      link.set_inner_text(&action.name);
      link.set_attribute("href", "#").map_err(js_to_anyhow)?;

      // Actions that change locations look different
      if action.transition.is_some() {
        link.set_class_name("location_change");
      }

      // Setup the callback that will be run if the action is selected.
      let callback_data = data_ptr.clone();
      let callback_action = action.clone();
      let callback = Closure::<dyn FnMut()>::new(move || {
        let mut data = callback_data.lock().unwrap();

        // Give the player all items the action yields
        for item in &callback_action.yields {
          data.inventory.insert(item.clone());
        }

        // Play music if requested
        if let Some(music) = &callback_action.music {
          if let Some(player) = data.music.get(music) {
            player.play();
          }
        }

        data.current_text = callback_action.text.clone();
        if !data.current_text.is_empty() {
          data.current_text += "<br/><br/>";
        }

        if let Some(destination) = &callback_action.transition {
          Self::goto_room(&mut data, callback_data.clone(), destination);
        } else {
          let room = match data.rooms.get(&data.current_room) {
            Some(r) => r.clone(),
            None => {
              log::error!(
                "Current room is set to nonexistant room {}, can't reload the actions",
                &data.current_room
              );
              return;
            }
          };
          data.current_text += &room.description;

          data.text_element.set_inner_html(&data.current_text);

          // Item states might have changed
          if let Err(err) = Self::update_actions(&mut data, callback_data.clone(), &room.actions) {
            log::error!("Unable to update the actions: {err:#}");
          }
        }

        // Save the new state
        Self::save(&mut data);
      });

      link.set_onclick(Some(callback.as_ref().unchecked_ref()));
      callback.forget();

      data
        .actions_element
        .append_child(&link)
        .map_err(js_to_anyhow)?;
    }

    Ok(())
  }

  fn save(data: &mut GameData) {
    let save = SaveGame {
      current_text: data.current_text.clone(),
      inventory: data.inventory.clone().into_iter().collect(),
      current_room: data.current_room.clone(),
    };

    let serialized = serde_json::to_string(&save).unwrap();

    let window = web_sys::window().unwrap();
    window
      .local_storage()
      .unwrap()
      .unwrap()
      .set("textadventure_save", &serialized)
      .unwrap();
  }

  fn load(data: &mut GameData, data_ptr: Rc<Mutex<GameData>>) {
    let window = web_sys::window().unwrap();
    if let Some(save) = window
      .local_storage()
      .unwrap()
      .unwrap()
      .get_item("textadventure_save")
      .unwrap()
    {
      let parsed = serde_json::from_str::<SaveGame>(&save);
      if let Ok(save) = parsed {
        data.inventory = save.inventory.into_iter().collect();

        Self::goto_room(data, data_ptr.clone(), &save.current_room);

        data.current_text = save.current_text;
        data.text_element.set_inner_html(&data.current_text);
      } else {
        log::warn!("Found malformed save data: {save}");
      }
    }
  }

  fn reset() {
    let window = web_sys::window().unwrap();
    window
      .local_storage()
      .unwrap()
      .unwrap()
      .remove_item("textadventure_save")
      .unwrap();

    window.location().set_href("/").unwrap();
  }
}

impl TryFrom<Adventure> for Game {
  type Error = anyhow::Error;

  fn try_from(value: Adventure) -> Result<Self, Self::Error> {
    // Load the music
    let mut music = HashMap::new();
    for (key, val) in value.assets.music {
      music.insert(key, SongPlayer::try_from(val)?);
    }

    let window = web_sys::window().ok_or(anyhow!("unable to get the window"))?;
    let document = window
      .document()
      .ok_or(anyhow!("unable to get the document"))?;

    let text_element: HtmlElement = document
      .get_element_by_id("maintext")
      .ok_or(anyhow!("Missing a #maintext element in the dom"))?
      .dyn_into()
      .map_err(|_| anyhow!("maintext ist not an html element"))?;

    let actions_element: HtmlElement = document
      .get_element_by_id("actions")
      .ok_or(anyhow!("Missing an #actions element in the dom"))?
      .dyn_into()
      .map_err(|_| anyhow!("maintext ist not an html element"))?;

    let reset_element: HtmlElement = document
      .get_element_by_id("reset")
      .ok_or(anyhow!("Missing a #reset element in the dom"))?
      .dyn_into()
      .map_err(|_| anyhow!("reset ist not an html element"))?;

    let reset_callback = Closure::<dyn FnMut()>::new(Self::reset);
    reset_element.set_onclick(Some(reset_callback.as_ref().unchecked_ref()));
    reset_callback.forget();

    let data = GameData {
      intro: value.intro,
      rooms: value.rooms,
      start: value.start,
      inventory: HashSet::new(),
      current_text: String::default(),
      current_room: String::default(),
      music,
      text_element,
      actions_element,
    };

    Ok(Self {
      data: Rc::new(Mutex::new(data)),
    })
  }
}

fn js_to_anyhow(val: JsValue) -> anyhow::Error {
  anyhow!("{val:?}")
}

#[derive(Serialize, Deserialize)]
struct SaveGame {
  current_text: String,
  inventory: Vec<String>,
  current_room: String,
}
