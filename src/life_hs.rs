use std::collections::{HashMap, HashSet};
use egui::{Color32, Ui, Vec2};

const BOARD_SIZE: u32 = 1024;
const BOARD_LAST: u32 = BOARD_SIZE + 1;
pub(crate) const BOARD_DISPLAY_SIZE: u32 = 512;
const CELL_DISPLAY_SIZE: f32 = (BOARD_DISPLAY_SIZE as f32) / (BOARD_SIZE as f32);
const CELL_PIXEL_SIZE: f32 = if CELL_DISPLAY_SIZE < 1.0 { 1.0 } else { CELL_DISPLAY_SIZE };
const CELLS_PER_PIXEL: u32 = if 1.0 / CELL_DISPLAY_SIZE < 1.0 { 1.0 } else { 1.0 / CELL_DISPLAY_SIZE } as u32;

#[derive(Clone, Debug, Default)]
pub(crate) struct LifeBoard {
    active: HashSet<(u32, u32)>,
    neighbors: HashSet<(u32, u32)>,
}

impl LifeBoard {
    pub(crate) fn new_checkerboard() -> Self {
        let active = (1u32..BOARD_LAST)
            .map(|x| (1u32..BOARD_LAST).map(move |y| (x, y)))
            .flatten()
            .filter(|(x, y)| (x + y) % 2 == 0)
            .collect::<HashSet<_>>();
        let neighbors = find_neighbor_cells(&active);
        Self { active, neighbors }
    }

    pub(crate) fn draw(&self, ui: &Ui) {
        let painter = ui.painter();
        let top_left = ui.min_rect().min;

        let mut display_pixels: HashMap<(u32, u32), f32> = HashMap::new();
        for (x, y) in self.active.iter() {
            *(
                display_pixels.entry((
                    x / CELLS_PER_PIXEL - 1,
                    y / CELLS_PER_PIXEL - 1,
                ))
                    .or_insert(0.0)
            ) += 1.0;
        }

        for ((x, y), c) in display_pixels {
            painter.rect_filled(
                egui::Rect::from([
                    top_left + Vec2::new(x as f32, y as f32) * CELL_PIXEL_SIZE,
                    top_left + Vec2::new((x + 1) as f32, (y + 1) as f32) * CELL_PIXEL_SIZE,
                ]),
                egui::Rounding::none(),
                Color32::from_rgb(egui::color::gamma_u8_from_linear_f32(
                    c / (CELLS_PER_PIXEL as f32).powf(2.0),
                ), 0, 0),
            )
        }
    }

    pub(crate) fn update(&mut self) {
        let mut new = self.active.clone();
        for cell in self.active.iter() {
            let count = get_neighbors(cell)
                .into_iter()
                .filter(|pos| self.active.contains(pos))
                .count();
            if count < 2 || count > 3 {
                new.remove(cell);
            }
        }

        for cell in self.neighbors.iter() {
            let count = get_neighbors(cell)
                .into_iter()
                .filter(|pos| self.active.contains(pos))
                .count();

            if count == 3 {
                new.insert(*cell);
            }
        }

        self.active = new;
        self.neighbors = find_neighbor_cells(&self.active);
    }
}

fn get_neighbors(pos: &(u32, u32)) -> [(u32, u32); 8] {
    let (x, y) = *pos;
    [
        (x - 1, y - 1),
        (x, y - 1),
        (x + 1, y - 1),
        (x + 1, y),
        (x + 1, y + 1),
        (x, y + 1),
        (x - 1, y + 1),
        (x - 1, y),
    ]
}

fn is_edge(pos: &(u32, u32)) -> bool {
    pos.0 == 0 || pos.1 == 0 || pos.0 == BOARD_LAST || pos.1 == BOARD_LAST
}

fn find_neighbor_cells(active: &HashSet<(u32, u32)>) -> HashSet<(u32, u32)> {
    let mut neighbors = active.iter()
        .map(|cell| get_neighbors(cell))
        .flatten()
        .collect::<HashSet<_>>();
    neighbors.retain(|pos| !is_edge(pos) && !active.contains(pos));
    neighbors
}
