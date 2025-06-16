use serde::{Deserialize, Serialize};
use crate::deserialize::paths::GMPath;
use crate::export_mod::export::{ModExporter, ModRef};
use crate::export_mod::unordered_list::{export_changes_unordered_list, EditUnorderedList};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModPath {
    pub name: ModRef,
    pub is_smooth: bool,
    pub is_closed: bool,
    pub precision: u32,
    pub points: Vec<ModPathPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModPathPoint {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
}


impl ModExporter<'_, '_> {
    pub fn export_paths(&self) -> Result<EditUnorderedList<ModPath, ModPath>, String> {
        export_changes_unordered_list(
            &self.original_data.paths.paths_by_index,
            &self.modified_data.paths.paths_by_index,
            |i| self.convert_path(i),
            |_, m| self.convert_path(m),    // force override all fields if path is edited (merging will not work properly)
            false,
        )
    }
    
    fn convert_path(&self, i: &GMPath) -> Result<ModPath, String> {
        Ok(ModPath {
            name: self.convert_string_ref(&i.name)?,
            is_smooth: i.is_smooth,
            is_closed: i.is_closed,
            precision: i.precision,
            points: i.points.iter().map(|i| ModPathPoint {
                x: i.x,
                y: i.y,
                speed: i.speed,
            }).collect(),
        })
    }
}

