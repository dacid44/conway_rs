use std::fs::File;
use std::io::{BufReader, BufRead};
use ca_formats::rle::Rle;
use egui::{Color32, Pos2, Ui, Vec2};
use ndarray::{Array1, Array2, Axis, s};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

const REAL_BOARD_SIZE: usize = 1024;
const BOARD_SIZE: usize = REAL_BOARD_SIZE + 2;
const BOARD_LAST: usize = REAL_BOARD_SIZE + 1;
pub(crate) const DISPLAY_SIZE: f32 = 512.0;
pub(crate) const CELL_SIZE: f32 = DISPLAY_SIZE / (REAL_BOARD_SIZE as f32);

pub(crate) fn draw_board(ui: &mut Ui, board: &Array2<bool>) {
    let painter = ui.painter();
    for ((x, y), _) in board.indexed_iter().filter(|(_, x)| **x) {
        if is_edges(x, y) {
            continue;
        }

        let top_left = ui.min_rect().min;
        painter.rect_filled(
            egui::Rect::from([
                top_left + Vec2::new((x - 1) as f32, (y - 1) as f32) * CELL_SIZE,
                top_left + Vec2::new(x as f32, y as f32) * CELL_SIZE,
            ]),
            egui::Rounding::none(),
            Color32::RED,
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn update_board(board: &mut Array2<bool>) {
    let mut new_board = new_blank_board();

    ndarray::Zip::from(new_board.slice_mut(s![1..-1, 1..-1]))
        .and(board.windows((3, 3)))
        .par_for_each(|a, b| {
            *a = match (b[[1, 1]], b.iter().filter(|c| **c).count() - (b[[1, 1]] as usize)) {
                (true, 2) => true,
                (true, 3) => true,
                (false, 3) => true,
                _ => false,
            }
        });

    *board = new_board;
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn update_board(board: &mut Array2<bool>) {
    let mut new_board = new_blank_board();

    ndarray::Zip::from(new_board.slice_mut(s![1..-1, 1..-1]))
        .and(board.windows((3, 3)))
        .for_each(|a, b| {
            *a = match (b[[1, 1]], b.iter().filter(|c| **c).count() - (b[[1, 1]] as usize)) {
                (true, 2) => true,
                (true, 3) => true,
                (false, 3) => true,
                _ => false,
            }
        });

    *board = new_board;
}

pub(crate) fn edit_board(board: &mut Array2<bool>, pos: Vec2) {
    let cell = &mut board[[
        (pos.x / CELL_SIZE).ceil() as usize,
        (pos.y / CELL_SIZE).ceil() as usize,
    ]];
    *cell = !*cell;
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum Shift {
    Up(usize),
    Down(usize),
    Left(usize),
    Right(usize),
}

pub(crate) fn shift_board(board: &mut Array2<bool>, shift: Shift) {
    let mut new_board = new_blank_board();
    match shift {
        Shift::Up(delta) => {
            new_board.lanes_mut(Axis(0))
                .into_iter()
                .skip(1)
                .zip(
                    board.lanes(Axis(0))
                        .into_iter()
                        .skip(delta + 1)
                )
                .for_each(|(mut a, b)| a.assign(&b));
        },
        Shift::Down(delta) => {
            new_board.lanes_mut(Axis(0))
                .into_iter()
                .skip(delta + 1)
                .zip(
                    board.lanes(Axis(0))
                        .into_iter()
                        .skip(1)
                )
                .for_each(|(mut a, b)| a.assign(&b));
        }
        Shift::Left(delta) => {
            new_board.lanes_mut(Axis(1))
                .into_iter()
                .skip(1)
                .zip(
                    board.lanes(Axis(1))
                        .into_iter()
                        .skip(delta + 1)
                )
                .for_each(|(mut a, b)| a.assign(&b));
        }
        Shift::Right(delta) => {
            new_board.lanes_mut(Axis(1))
                .into_iter()
                .skip(delta + 1)
                .zip(
                    board.lanes(Axis(1))
                        .into_iter()
                        .skip(1)
                )
                .for_each(|(mut a, b)| a.assign(&b));
        }
    }

    *board = new_board;
}

pub(crate) fn import_from_file() -> Array2<bool> {
    let file = futures::executor::block_on(
        rfd::AsyncFileDialog::new()
            .pick_file()
    );

    let mut board = new_blank_board();

    board.lanes_mut(Axis(0))
        .into_iter()
        .skip(1)
        .zip(
            (&futures::executor::block_on(
                file.unwrap().read()
            )[..])
                .lines()
                .flatten()
                .filter(|line| !line.starts_with("!"))
        )
        .for_each(|(mut row, line)| {
            row.iter_mut()
                .skip(1)
                .zip(line.chars())
                .for_each(|(cell, c)| *cell = c == 'O')
        });

    board.row_mut(0).assign(&new_blank_line());
    board.row_mut(BOARD_LAST).assign(&new_blank_line());
    board.column_mut(0).assign(&new_blank_line());
    board.column_mut(BOARD_LAST).assign(&new_blank_line());

    board
}

pub(crate) fn import_rle() -> Array2<bool> {

    let file = futures::executor::block_on(
        rfd::AsyncFileDialog::new()
            .add_filter("RLE", &["rle"])
            .pick_file()
    );

    let mut board = new_blank_board();

    for c in Rle::new_from_file(
        &futures::executor::block_on(file.unwrap().read())[..]
    )
        .unwrap()
        .map(|c| c.unwrap().position)
    {
        board[[c.0 as usize + 1, c.1 as usize + 1]] = true;
    }

    board
}

fn is_edges(x: usize, y: usize) -> bool {
    x == 0 || y == 0 || x == BOARD_LAST || y == BOARD_LAST
}

pub(crate) fn new_blank_board() -> Array2<bool> {
    Array2::default((BOARD_SIZE, BOARD_SIZE))
}

pub(crate) fn new_blank_line() -> Array1<bool> {
    Array1::default(BOARD_SIZE)
}

pub(crate) fn new_checkerboard() -> Array2<bool> {
    let mut board = new_blank_board();
    for ((x, y), val) in board.indexed_iter_mut() {
        if is_edges(x, y) {
            continue;
        }

        if (x + y) % 2 == 0 {
            *val = true;
        }
    }
    board
}
