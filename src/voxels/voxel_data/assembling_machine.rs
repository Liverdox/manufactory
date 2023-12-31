use std::sync::{Arc, Mutex};

use crate::{recipes::{item::{PossibleItem, Item}, storage::Storage, recipe::{Recipe, ActiveRecipe}, recipes::RECIPES}, world::global_coords::GlobalCoords, gui::{draw::Draw, my_widgets::{assembling_machine_slot::assembling_machine_slot, recipe::recipe}}, player::inventory::PlayerInventory, engine::texture::TextureAtlas, bytes::{BytesCoder, AsFromBytes, cast_bytes_from_slice, cast_vec_from_bytes}};
use crate::gui::my_widgets::container::container;

use super::{multiblock::MultiBlock, DrawStorage};

const INGREDIENT_LENGTH: usize = 3;
const RESULT_LENGTH: usize = 1;
const TOTAL_LENGTH: usize = INGREDIENT_LENGTH+RESULT_LENGTH;


#[derive(Debug)]
pub struct AssemblingMachine {
    storage: [PossibleItem; TOTAL_LENGTH],

    structure_coordinates: Vec<GlobalCoords>,

    selected_recipe: Option<&'static Recipe>,
    active_recipe: Option<ActiveRecipe>,
}


impl AssemblingMachine {
    pub fn new(structure_coordinates: Vec<GlobalCoords>) -> Self {
        Self {
            storage: [PossibleItem::new_none(); TOTAL_LENGTH],
            structure_coordinates,
            selected_recipe: None,
            active_recipe: None,
        }
    }

    pub fn selected_recipe(&self) -> Option<&'static Recipe> {
        self.selected_recipe
    }

    pub fn select_recipe(&mut self, index: usize) -> ([PossibleItem; TOTAL_LENGTH], Vec<Item>) {
        self.selected_recipe = Some(&RECIPES().all[index]);
        let mut result = [PossibleItem::new_none(); TOTAL_LENGTH];
        std::mem::swap(&mut result, &mut self.storage);
        let ingredients = self.active_recipe.take().map_or(vec![], |ac| ac.recipe.ingredients);
        (result, ingredients)
    }

    pub fn update(&mut self) {
        if self.active_recipe.is_none() && self.selected_recipe.is_some() {
            self.active_recipe = self.start_recipe(self.selected_recipe.unwrap());
        }

        let Some(active_recipe) = &self.active_recipe else {return};
        if !active_recipe.is_finished() || !self.storage()[3].is_possible_add(&active_recipe.recipe.result) {return};
        
        let add_item = active_recipe.recipe.result;
        self.mut_storage()[3].try_add_item(&add_item);
        self.active_recipe = None;
    }
}



impl Storage for AssemblingMachine {
    fn storage(&self) -> &[PossibleItem] {
        &self.storage
    }

    fn mut_storage(&mut self) -> &mut [PossibleItem] {
        &mut self.storage
    }

    fn is_item_exist(&self, item: &Item) -> bool {
        self.storage()[0..INGREDIENT_LENGTH]
            .iter()
            .map(|possible_item| possible_item.contains(item.id()))
            .sum::<u32>() >= item.count
    }

    fn remove(&mut self, item: &Item) -> Option<Item> {
        let mut sub_item = Item::from(item);
        for possible_item in self.mut_storage()[0..INGREDIENT_LENGTH].iter_mut() {
            let remainder = possible_item.try_sub_item(&sub_item);
            let Some(remainder) = remainder else {return None};
            sub_item = remainder;
        }
        Some(sub_item)
    }

    fn add(&mut self, item: &Item, _: bool) -> Option<Item> {
        let mut added_item = Item::from(item);
        let Some(recipe) = self.selected_recipe else {return Some(added_item)};
        for (index, possible_item) in self.mut_storage()[0..INGREDIENT_LENGTH].iter_mut().enumerate() {
            if recipe.ingredients.get(index).map(|i| i.id()) == Some(item.id()) {
                let remainder = possible_item.try_add_item(&added_item);
                let Some(remainder) = remainder else {return None};
                added_item = remainder;
            }
        }
        Some(added_item)
    }

    fn take_first_existing(&mut self, max_count: u32) -> Option<(Item, usize)> {
        for (i, possible_item) in self.mut_storage()[INGREDIENT_LENGTH..TOTAL_LENGTH].iter_mut().enumerate() {
            let Some(item) = possible_item.try_take(max_count) else {continue};
            return Some((item, i))
        }
        None
    }
}


impl MultiBlock for AssemblingMachine {
    fn structure_coordinates(&self) -> &[GlobalCoords] {
        &self.structure_coordinates
    }

    fn mut_structure_coordinates(&mut self) -> &mut [GlobalCoords] {
        &mut self.structure_coordinates
    }
}


impl Draw for AssemblingMachine {
    fn draw(&mut self, ui: &mut egui::Ui, atlas: Arc<TextureAtlas>, inventory: Arc<Mutex<PlayerInventory>>) {
        let mut task: Option<usize> = None;
        let selected_recipe = self.selected_recipe();
        if let Some(selected_recipe) = selected_recipe {
            ui.horizontal(|ui| {
                for (i, item) in self.storage().iter().enumerate() {
                    if ui.add(assembling_machine_slot(&atlas, item, i, selected_recipe, i==3)).drag_started() {
                        task = Some(i);
                    };
                }
            });
        }
        ui.vertical(|ui| {
            ui.add(container(|ui| {
                let style = egui::Style {
                    spacing: egui::style::Spacing { item_spacing: egui::vec2(8.0, 8.0), ..Default::default() },
                    ..Default::default()
                };
                ui.set_style(style);
                ui.horizontal(|ui| {
                    for i in RECIPES().assembler.all() {
                        if ui.add(recipe(&atlas, i)).drag_started() {
                            let result = self.select_recipe(i.index);
                            for item in result.0 {
                                let Some(item) = item.0 else {continue};
                                inventory.lock().unwrap().add(&item, true);
                            }
                            for item in result.1 {
                                inventory.lock().unwrap().add(&item, true);
                            }
                        };
                    }
                });
            }, None));
        });

        if let Some(task) = task {
            let Some(item) = self.mut_storage()[task].0.take() else {return};
            let remainder = inventory.lock().unwrap().add(&item, true);
            if let Some(r) = remainder {self.set(&r, task)}
        }
    }
}

impl DrawStorage for AssemblingMachine {}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Header {
    selected_recipe_id: u32,
    active_recipe_id: u32,
    storage_len: u32,
    structure_len: u32,
}
impl AsFromBytes for Header {}

impl BytesCoder for AssemblingMachine {
    fn decode_bytes(bytes: &[u8]) -> Self {
        let header = Header::from_bytes(&bytes[0..Header::size()]);
        let selected_recipe = if header.selected_recipe_id != u32::MAX {
            Some(&RECIPES().all[header.selected_recipe_id as usize])
        } else {
            None
        };
        let active_recipe = if header.active_recipe_id != u32::MAX {
            Some(RECIPES().all[header.active_recipe_id as usize].start_absolute())
        } else {
            None
        };
        let storage_size = Header::size() + header.storage_len as usize;
        let storage = <[PossibleItem; TOTAL_LENGTH]>::decode_bytes(&bytes[Header::size()..storage_size]);
        let structure_size = storage_size+header.structure_len as usize;
        let structure = cast_vec_from_bytes(&bytes[storage_size..structure_size]);

        Self {
            selected_recipe,
            active_recipe,
            storage,
            structure_coordinates: structure,
        }
    }
    fn encode_bytes(&self) -> Box<[u8]> {
        let mut bytes = Vec::new();

        let storage = self.storage.encode_bytes();
        let structure = cast_bytes_from_slice(&self.structure_coordinates);
        bytes.extend(Header {
            selected_recipe_id: self.selected_recipe.map(|r| r.id).unwrap_or(u32::MAX),
            active_recipe_id: self.active_recipe.as_ref().map(|r| r.recipe.id).unwrap_or(u32::MAX),
            storage_len: storage.len() as u32,
            structure_len: structure.len() as u32,
        }.as_bytes());
        bytes.extend(storage.as_ref());
        bytes.extend(structure);
        bytes.into()
    }
}