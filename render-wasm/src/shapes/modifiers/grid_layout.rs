#![allow(dead_code, unused_variables)]
use crate::math::{intersect_rays, Bounds, Matrix, Point, Ray, Vector, VectorExt};
use crate::shapes::{GridCell, GridData, GridTrack, GridTrackType, LayoutData, Modifier, Shape};
use crate::uuid::Uuid;
use std::collections::{HashMap, VecDeque};

use super::common::GetBounds;

const MIN_SIZE: f32 = 0.01;
const MAX_SIZE: f32 = f32::INFINITY;

#[derive(Debug)]
struct CellData<'a> {
    shape: &'a Shape,
    anchor: Point,
    width: f32,
    height: f32,
}

#[derive(Debug)]
struct TrackData {
    track_type: GridTrackType,
    value: f32,
    size: f32,
    max_size: f32,
    anchor_start: Point,
    anchor_end: Point,
}

fn calculate_tracks(
    is_column: bool,
    layout_data: &LayoutData,
    grid_data: &GridData,
    layout_bounds: &Bounds,
    cells: &Vec<GridCell>,
    shapes: &HashMap<Uuid, Shape>,
    bounds: &HashMap<Uuid, Bounds>,
) -> Vec<TrackData> {
    let layout_size = if is_column {
        layout_bounds.width() - layout_data.padding_left - layout_data.padding_right
    } else {
        layout_bounds.height() - layout_data.padding_top - layout_data.padding_bottom
    };

    let grid_tracks = if is_column {
        &grid_data.columns
    } else {
        &grid_data.rows
    };

    let mut tracks = init_tracks(grid_tracks, layout_size);
    set_auto_base_size(is_column, &mut tracks, cells, shapes, bounds);
    set_auto_multi_span(is_column, layout_data, &layout_bounds, &mut tracks);
    set_flex_multi_span(is_column, layout_data, &layout_bounds, &mut tracks);
    set_fr_value(is_column, layout_data, &layout_bounds, &mut tracks);
    stretch_tracks(is_column, layout_data, &layout_bounds, &mut tracks);
    assign_anchors(is_column, layout_data, &layout_bounds, &mut tracks);
    return tracks;
}

fn init_tracks(track: &Vec<GridTrack>, size: f32) -> Vec<TrackData> {
    track
        .iter()
        .map(|t| {
            let (size, max_size) = match t.track_type {
                GridTrackType::Fixed => (t.value, t.value),
                GridTrackType::Percent => (size * t.value / 100.0, size * t.value / 100.0),
                _ => (MIN_SIZE, MAX_SIZE),
            };
            TrackData {
                track_type: t.track_type,
                value: t.value,
                size,
                max_size,
                anchor_start: Point::default(),
                anchor_end: Point::default(),
            }
        })
        .collect()
}

// Go through cells to adjust auto sizes for span=1. Base is the max of its children
fn set_auto_base_size(
    column: bool,
    tracks: &mut Vec<TrackData>,
    cells: &Vec<GridCell>,
    shapes: &HashMap<Uuid, Shape>,
    bounds: &HashMap<Uuid, Bounds>,
) {
    for cell in cells {
        let (prop, prop_span) = if column {
            (cell.column, cell.column_span)
        } else {
            (cell.row, cell.row_span)
        };

        if prop_span != 1 {
            continue;
        }

        let track = &mut tracks[prop as usize];

        if track.track_type != GridTrackType::Auto && track.track_type != GridTrackType::Flex {
            continue;
        }

        let Some(shape) = cell.shape.and_then(|id| shapes.get(&id)) else {
            continue;
        };

        let bounds = bounds.find(shape);

        let shape_size = if column {
            bounds.width()
        } else {
            bounds.height()
        };

        let min_size = if column && shape.is_layout_horizontal_fill() {
            shape.layout_item.and_then(|i| i.min_w).unwrap_or(MIN_SIZE)
        } else if !column && shape.is_layout_vertical_fill() {
            shape.layout_item.and_then(|i| i.min_h).unwrap_or(MIN_SIZE)
        } else {
            shape_size
        };

        track.size = f32::max(track.size, min_size);
    }
}

// Adjust multi-spaned cells with no flex columns
fn set_auto_multi_span(
    _column: bool,
    _layout_data: &LayoutData,
    _layout_bounds: &Bounds,
    _tracks: &mut Vec<TrackData>,
) {
    // Ordena descendente por prop-span
    // Quitamos los tracks que tengan flex (se reservaran en el otro metodo)
    // Recuperamos el valor que tenemos que distribuir (el tamaño minimo de la celda restando los gaps del span)
    // Distribuimos el tamaño entre los tracks que ya tienen valor fijo
    // Distribuimos el espacio entre los "auto"
    // Si aún tenemos espacio dividimos entre todos los tracks (hay que reservar suficiente espacio)
}

fn set_flex_multi_span(
    _column: bool,
    _layout_data: &LayoutData,
    _layout_bounds: &Bounds,
    _tracks: &mut Vec<TrackData>,
) {
    // Ordena descendente por prop-span
    // Mirar que alguna de sus tracks es flex
    // Recuperamos el valor que tenemos que distribuir (el tamaño minimo de la celda restando los gaps del span)
    // Distribuimos el tamaño primero por los tracks que ya tienen tamaño fijo
    // Cuando hemos distribuido dividimos entre los frs en partes iguales con el resto
}

// Calculate the `fr` unit and adjust the size
fn set_fr_value(
    _column: bool,
    _layout_data: &LayoutData,
    _layout_bounds: &Bounds,
    _tracks: &mut Vec<TrackData>,
) {
    // Calculamos el numero de FR's que tenemos que distribuir
    // Dividimos el espacio restante entre los FRS
    // Asignamos el espacio alos FRS
}

fn stretch_tracks(
    _column: bool,
    _layout_data: &LayoutData,
    _layout_bounds: &Bounds,
    _tracks: &mut Vec<TrackData>,
) {
    // Si estamos en stretch distribuimos el espacio que tenemos sobrante entre los tracks auto
}

fn assign_anchors(
    column: bool,
    layout_data: &LayoutData,
    layout_bounds: &Bounds,
    tracks: &mut Vec<TrackData>,
) {
    let mut cursor = layout_bounds.nw;

    let (v, gap, padding_start) = if column {
        (
            layout_bounds.hv(1.0),
            layout_data.row_gap,
            layout_data.padding_left,
        )
    } else {
        (
            layout_bounds.vv(1.0),
            layout_data.column_gap,
            layout_data.padding_top,
        )
    };

    cursor = cursor + (v * padding_start);

    for track in tracks {
        track.anchor_start = cursor;
        track.anchor_end = cursor + (v * track.size);
        cursor = track.anchor_end + (v * gap);
    }
}

fn cell_bounds(
    layout_bounds: &Bounds,
    column_start: Point,
    column_end: Point,
    row_start: Point,
    row_end: Point,
) -> Option<Bounds> {
    let hv = layout_bounds.hv(1.0);
    let vv = layout_bounds.vv(1.0);
    let nw = intersect_rays(&Ray::new(column_start, vv), &Ray::new(row_start, hv))?;
    let ne = intersect_rays(&Ray::new(column_end, vv), &Ray::new(row_start, hv))?;
    let sw = intersect_rays(&Ray::new(column_start, vv), &Ray::new(row_end, hv))?;
    let se = intersect_rays(&Ray::new(column_end, vv), &Ray::new(row_end, hv))?;
    Some(Bounds::new(nw, ne, se, sw))
}

fn create_cell_data<'a>(
    layout_bounds: &Bounds,
    shapes: &'a HashMap<Uuid, Shape>,
    cells: &Vec<GridCell>,
    column_tracks: &Vec<TrackData>,
    row_tracks: &Vec<TrackData>,
) -> Vec<CellData<'a>> {
    let mut result = Vec::<CellData<'a>>::new();

    for cell in cells {
        let Some(shape_id) = cell.shape else {
            continue;
        };
        let Some(shape) = shapes.get(&shape_id) else {
            continue;
        };

        let column_start = (cell.column - 1) as usize;
        let column_end = (cell.column + cell.column_span - 2) as usize;
        let row_start = (cell.row - 1) as usize;
        let row_end = (cell.row + cell.row_span - 2) as usize;
        let Some(cell_bounds) = cell_bounds(
            layout_bounds,
            column_tracks[column_start].anchor_start,
            column_tracks[column_end].anchor_end,
            row_tracks[row_start].anchor_start,
            row_tracks[row_end].anchor_end,
        ) else {
            continue;
        };

        result.push(CellData {
            shape,
            anchor: cell_bounds.nw,
            width: cell_bounds.width(),
            height: cell_bounds.height(),
        });
    }

    result
}

fn calculate_cell_data<'a>(
    shape: &Shape,
    layout_data: &LayoutData,
    grid_data: &GridData,
    shapes: &'a HashMap<Uuid, Shape>,
    bounds: &HashMap<Uuid, Bounds>,
) -> Vec<CellData<'a>> {
    let result: Vec<CellData<'a>> = vec![];

    let layout_bounds = bounds.find(shape);

    let column_tracks = calculate_tracks(
        true,
        layout_data,
        grid_data,
        &layout_bounds,
        &grid_data.cells,
        shapes,
        bounds,
    );

    let row_tracks = calculate_tracks(
        false,
        layout_data,
        grid_data,
        &layout_bounds,
        &grid_data.cells,
        shapes,
        bounds,
    );

    create_cell_data(
        &layout_bounds,
        shapes,
        &grid_data.cells,
        &column_tracks,
        &row_tracks,
    )
}

fn child_position(child_bounds: &Bounds, cell: &CellData) -> Point {
    cell.anchor
}

pub fn reflow_grid_layout<'a>(
    shape: &Shape,
    layout_data: &LayoutData,
    grid_data: &GridData,
    shapes: &'a HashMap<Uuid, Shape>,
    bounds: &HashMap<Uuid, Bounds>,
) -> VecDeque<Modifier> {
    let mut result = VecDeque::new();

    let cells = calculate_cell_data(shape, layout_data, grid_data, shapes, bounds);

    for cell in cells.iter() {
        let child = cell.shape;
        let child_bounds = bounds.find(child);
        let position = child_position(&child_bounds, cell);

        let mut transform = Matrix::default();
        let delta_v = Vector::new_points(&child_bounds.nw, &position);

        if delta_v.x.abs() > MIN_SIZE || delta_v.y.abs() > MIN_SIZE {
            transform.post_concat(&Matrix::translate(delta_v));
        }

        result.push_back(Modifier::transform(child.id, transform));
    }

    result
}
