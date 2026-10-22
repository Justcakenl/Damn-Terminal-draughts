use std::{fs, io, path::PathBuf};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use crate::game::state::GameState;

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
struct SaveFile { version: u8, state: GameState }

#[allow(dead_code)]
pub fn path() -> Option<PathBuf> {
    ProjectDirs::from("org", "damn", "DAMN").map(|d| d.data_dir().join("save.json"))
}
#[allow(dead_code)]
pub fn save(state: &GameState) -> io::Result<()> {
    let target = path().ok_or_else(|| io::Error::other("cannot determine data directory"))?;
    if let Some(parent) = target.parent() { fs::create_dir_all(parent)?; }
    let tmp = target.with_extension("tmp");
    let bytes = serde_json::to_vec_pretty(&SaveFile { version: 1, state: state.clone() }).map_err(io::Error::other)?;
    fs::write(&tmp, bytes)?;
    fs::rename(tmp, target)
}
#[allow(dead_code)]
pub fn load() -> io::Result<GameState> {
    let file = fs::read(path().ok_or_else(|| io::Error::other("cannot determine data directory"))?)?;
    let saved: SaveFile = serde_json::from_slice(&file).map_err(io::Error::other)?;
    if saved.version != 1 { return Err(io::Error::other("unsupported save version")); }
    Ok(saved.state)
}
