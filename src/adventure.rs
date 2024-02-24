use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Document {
    pub adventure: Adventure,
}

#[derive(Deserialize, Clone)]
pub struct Adventure {
    pub name: String,
    pub start: String,
    pub intro: String,
    pub rooms: HashMap<String, Room>,
    pub assets: AdventureAssets,
}

#[derive(Deserialize, Clone)]
pub struct AdventureAssets {
    pub music: HashMap<String, Song>,
}

#[derive(Deserialize, Clone)]
pub struct Song {
    pub unit_length: f64,
    pub voices: Vec<SongVoice>,
}

#[derive(Deserialize, Clone)]
pub struct SongVoice {
    pub instrument: String,
    pub notes: String,
}

#[derive(Deserialize, Clone)]
pub struct Room {
    pub description: String,
    pub actions: Vec<Action>,
}

#[derive(Deserialize, Clone, Default)]
pub struct Action {
    pub name: String,
    #[serde(default)]
    pub yields: Vec<String>,
    pub text: String,
    #[serde(default)]
    pub depends: ActionDependencies,
    #[serde(default)]
    pub transition: Option<String>,
    #[serde(default)]
    pub music: Option<String>,
}

#[derive(Deserialize, Clone, Default)]
pub struct ActionDependencies {
    #[serde(default)]
    pub not: Vec<String>,
    #[serde(default)]
    pub on: Vec<String>,
}
