use crate::{
    app::DevApp, draw, editor::serialize::Ppm, Button, Color, DrawContext, Sprite, State,
    SPRITE_HEIGHT, SPRITE_WIDTH,
};
use std::{fs::File, io::Write};

use self::canvas::Canvas;
mod canvas;
pub mod serialize;

pub struct SpriteEditor {
    mouse_x: i32,
    mouse_y: i32,
    mouse_pressed: bool,
    highlighted_color: Color,
    bottom_text: String,
    selected_sprite: u8,
    cursor_sprite: &'static Sprite,
    draw_mode: DrawMode,
}

#[derive(Debug)]
enum DrawMode {
    Pencil,
    Line(Option<LineState>),
}

#[derive(Debug)]
struct LineState {
    start: (i32, i32),
}

const CANVAS_X: i32 = 79; // end = 120
const CANVAS_Y: i32 = 10;

const SPRITES_PER_ROW: u8 = 16;

impl SpriteEditor {
    fn draw_tools(&self, draw_context: &mut DrawContext) {
        draw_context.palt(Some(0));

        match &self.draw_mode {
            DrawMode::Pencil => {
                draw_context.spr(14, 12, 78);
                draw_context.spr(31, 22, 78);
            }
            DrawMode::Line(_) => {
                draw_context.spr(15, 12, 78);
                draw_context.spr(30, 22, 78);
            }
        }
    }

    fn handle_draw_intent(&mut self, state: &mut State) {
        let sprite = &mut state
            .sprite_sheet
            .get_sprite_mut(self.selected_sprite.into())
            .sprite;

        match &mut self.draw_mode {
            DrawMode::Pencil => {
                if let Some((x, y)) = Canvas::try_lookup(self.mouse_x, self.mouse_y) {
                    self.bottom_text = format!("X:{} Y:{}", x, y);

                    if self.mouse_pressed {
                        sprite[(x + y * 8) as usize] = self.highlighted_color;
                    }
                }
            }
            DrawMode::Line(None) => {
                if self.mouse_pressed {
                    self.draw_mode = DrawMode::Line(Some(LineState {
                        start: (self.mouse_x, self.mouse_y),
                    }));
                }
            }
            DrawMode::Line(Some(LineState { start })) => {
                if !self.mouse_pressed {
                    // Draw the line when the mouse is released
                    let (start_x, start_y) = Canvas::to_local(start.0, start.1);
                    let (end_x, end_y) = Canvas::to_local(self.mouse_x, self.mouse_y);

                    for (x, y) in draw::line(start_x, start_y, end_x, end_y) {
                        sprite[(x + y * SPRITE_WIDTH as i32) as usize] = self.highlighted_color;
                    }

                    self.draw_mode = DrawMode::Line(None);
                } else {
                    if let Some((x, y)) = Canvas::try_lookup(self.mouse_x, self.mouse_y) {
                        self.bottom_text = format!("X:{} Y:{}", x, y);
                    };
                }
            }
        }
    }

    fn draw_sprite_sheet(&self, y_start: i32, draw_context: &mut DrawContext) {
        const BORDER: i32 = 1;
        const HEIGHT: i32 = 32;

        draw_context.line(0, y_start, 128, y_start, 0);

        for sprite_y in 0..4 {
            for sprite_x in 0..16 {
                let sprite_index = sprite_x + sprite_y * 16;

                draw_context.spr(
                    sprite_index,
                    (sprite_x * SPRITE_WIDTH) as i32,
                    y_start + BORDER as i32 + (sprite_y * SPRITE_WIDTH) as i32,
                )
            }
        }
        draw_context.line(
            0,
            y_start + HEIGHT + BORDER,
            128,
            y_start + HEIGHT + BORDER,
            0,
        );

        // Draw highlight of selected sprite
        // TODO: Clean this up, in particular, find a way not to repeat all of these calculations
        let selected_sprite_x = self.selected_sprite % SPRITES_PER_ROW;
        let selected_sprite_y = self.selected_sprite / SPRITES_PER_ROW;

        Rect {
            x: selected_sprite_x as i32 * SPRITE_WIDTH as i32,
            y: y_start + BORDER as i32 + (selected_sprite_y as usize * SPRITE_WIDTH) as i32,
            width: SPRITE_WIDTH as i32,
            height: SPRITE_HEIGHT as i32,
        }
        .highlight(draw_context, false, 7);
    }

    fn selected_sprite<'a>(&self, state: &'a State) -> &'a Sprite {
        state.sprite_sheet.get_sprite(self.selected_sprite.into())
    }
}

fn serialize(bytes: &[u8]) {
    let mut file = File::create("sprite_sheet.txt").unwrap();
    file.write_all(bytes).unwrap();
}

pub static MOUSE_SPRITE: &[Color] = &[
    0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 1, 0, 0, 0, 0, //
    0, 0, 1, 7, 1, 0, 0, 0, //
    0, 0, 1, 7, 7, 1, 0, 0, //
    0, 0, 1, 7, 7, 7, 1, 0, //
    0, 0, 1, 7, 7, 7, 7, 1, //
    0, 0, 1, 7, 7, 1, 1, 0, //
    0, 0, 0, 1, 1, 7, 1, 0, //
];

static MOUSE_TARGET_SPRITE: &[Color] = &[
    0, 0, 0, 1, 0, 0, 0, 0, //
    0, 0, 1, 7, 1, 0, 0, 0, //
    0, 1, 0, 0, 0, 1, 0, 0, //
    1, 7, 0, 0, 0, 7, 1, 0, //
    0, 1, 0, 0, 0, 1, 0, 0, //
    0, 0, 1, 7, 1, 0, 0, 0, //
    0, 0, 0, 1, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, //
];

impl DevApp for SpriteEditor {
    fn init() -> Self {
        // let mut sprite_sheet = vec![11; SPRITE_AREA * SPRITE_COUNT];

        Self {
            mouse_x: 64,
            mouse_y: 64,
            mouse_pressed: false,
            highlighted_color: 7,
            bottom_text: String::new(),
            selected_sprite: 0,
            cursor_sprite: Sprite::new(MOUSE_SPRITE),
            draw_mode: DrawMode::Line(None),
        }
    }

    fn update(&mut self, state: &mut State) {
        self.mouse_x = state.mouse_x;
        self.mouse_y = state.mouse_y;
        self.mouse_pressed = state.btn(Button::Mouse);
        self.bottom_text = String::new();
        self.cursor_sprite = Sprite::new(MOUSE_SPRITE);

        // Handle mouse over color palette
        for color in 0..16 {
            if color_position(color).contains(self.mouse_x, self.mouse_y) {
                self.bottom_text = format!("COLOUR {}", color);
                if self.mouse_pressed {
                    self.highlighted_color = color;
                }
                return;
            }
        }

        // Handle mouse over canvas
        // TODO: This might be bad (consider dragging mouse outside the canvas while drawing lines)
        if Canvas::position().contains(self.mouse_x, self.mouse_y) {
            self.cursor_sprite = Sprite::new(MOUSE_TARGET_SPRITE);

            self.handle_draw_intent(state);
        }

        // Handle mouse over sprite sheet
        if SPRITE_SHEET_AREA.contains(self.mouse_x, self.mouse_y) {
            self.bottom_text = "IN SPRITE SHEET".into();

            for x in 0..SPRITES_PER_ROW {
                // TODO: Use a const for the "4"
                for y in 0..4 {
                    let sprite_area = Rect {
                        x: x as i32 * SPRITE_WIDTH as i32,
                        y: SPRITE_SHEET_AREA.y + (y as usize * SPRITE_WIDTH) as i32,
                        width: SPRITE_WIDTH as i32,
                        height: SPRITE_HEIGHT as i32,
                    };

                    if sprite_area.contains(self.mouse_x, self.mouse_y) {
                        let sprite = x + y * SPRITES_PER_ROW;
                        self.bottom_text = format!("IN SPRITE {}", sprite);
                        if self.mouse_pressed {
                            self.selected_sprite = sprite;
                        }
                        break;
                    }
                }
            }
        }

        if state.btnp(Button::X) {
            println!("[Editor] Serializing sprite sheet");
            serialize(state.sprite_sheet.serialize().as_bytes());
            Ppm::from_sprite_sheet(&state.sprite_sheet)
                .write_file("./sprite_sheet.ppm")
                .expect("Couldn't write");
        }

        if state.btnp(Button::Down) {
            state
                .sprite_sheet
                .get_sprite_mut(self.selected_sprite.into())
                .shift_down();
        }

        if state.btnp(Button::Up) {
            state
                .sprite_sheet
                .get_sprite_mut(self.selected_sprite.into())
                .shift_up();
        }

        if state.btnp(Button::Left) {
            state
                .sprite_sheet
                .get_sprite_mut(self.selected_sprite.into())
                .shift_left();
        }

        if state.btnp(Button::Right) {
            state
                .sprite_sheet
                .get_sprite_mut(self.selected_sprite.into())
                .shift_right();
        }

        self.bottom_text = format!("{:?}", Canvas::to_local(self.mouse_x, self.mouse_y));

        if let DrawMode::Line(Some(LineState { start })) = self.draw_mode {
            self.bottom_text = format!("{:?}", start);
        }
    }

    fn draw(&self, draw_context: &mut DrawContext) {
        draw_context.cls();

        draw_context.palt(None);
        draw_context.rectfill(0, 0, 127, 127, 5);

        // Draw top menu bar
        draw_context.rectfill(0, 0, 127, 7, 8);

        // Draw bottom menu bar
        draw_context.rectfill(0, 121, 127, 127, 8);

        // draw canvas
        Canvas::position().fill(draw_context, 0);

        for x in 0..8 {
            for y in 0..8 {
                let color = self
                    .selected_sprite(&draw_context.state)
                    .pget(x as isize, y as isize);

                Canvas::pixel_rect(x, y).fill(draw_context, color);
            }
        }

        self.draw_tools(draw_context);

        // TODO: Look up correct positions
        draw_context.palt(Some(0));
        match &self.draw_mode {
            DrawMode::Pencil => {}
            DrawMode::Line(None) => {
                // println!("not drawing a line")
            }
            DrawMode::Line(Some(line_state)) => {
                let start = line_state.start;

                let (start_x, start_y) = Canvas::to_local(start.0, start.1);
                let (end_x, end_y) = Canvas::to_local(self.mouse_x, self.mouse_y);

                for (x, y) in draw::line(start_x, start_y, end_x, end_y) {
                    Canvas::pixel_rect(x as i32, y as i32)
                        .fill(draw_context, self.highlighted_color);
                }
            }
        }

        draw_context.palt(None);
        // let tools_area = Rect {
        //     x: 0,
        //     y: canvas_position().bottom() + 1,
        //     width: 128,
        //     height: 11,
        // };
        // tools_area.fill(draw_context, 12);

        let thumbnail_area = Rect {
            x: Canvas::position().right() - 2,
            y: Canvas::position().bottom() + 3,
            width: 8,
            height: 8,
        };

        thumbnail_area.fill(draw_context, 9);
        draw_context.spr(
            self.selected_sprite as usize,
            thumbnail_area.x,
            thumbnail_area.y,
        );

        Rect {
            x: thumbnail_area.right() + 2,
            y: thumbnail_area.y + 1,
            width: 13,
            height: 7,
        }
        .fill(draw_context, 6);
        let selected_sprite_str = format!("{:0width$}", self.selected_sprite, width = 3);
        draw_context.print(
            &selected_sprite_str,
            thumbnail_area.right() + 3,
            thumbnail_area.y + 2,
            13,
        );

        // Draw sprite sheet
        // TODO: Remove this, just here to make sure I'm not displaying the sprite sheet incorrectly
        SPRITE_SHEET_AREA.fill(draw_context, 2);

        self.draw_sprite_sheet(SPRITE_SHEET_AREA.y, draw_context);

        // Draw color palette
        draw_context.rectfill(
            CANVAS_X,
            CANVAS_Y,
            CANVAS_X + BOX_SIZE - 1,
            CANVAS_Y + BOX_SIZE - 1,
            0,
        );

        for color in 0..16 {
            let Rect {
                x,
                y,
                width,
                height,
            } = color_position(color);

            draw_context.rectfill(x, y, x + width - 1, y + height - 1, color as u8);
        }

        color_position(self.highlighted_color).highlight(draw_context, true, 7);

        draw_context.print(&self.bottom_text, 1, 122, 2);

        // print_debug_strings(draw_context, 10, 100);

        // Render page buttons

        // Always render the mouse last (on top of everything)
        draw_context.palt(Some(0));
        draw_context.raw_spr(self.cursor_sprite, self.mouse_x - 3, self.mouse_y - 2);
    }
}

const SPRITE_SHEET_AREA: Rect = Rect {
    x: 0,
    y: 87,
    width: 128,
    height: 34,
};

const SIZE: i32 = 10;
const BOX_SIZE: i32 = 4 * SIZE + 2;

#[derive(Debug)]
struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Rect {
    pub fn contains(&self, x: i32, y: i32) -> bool {
        let contains_x = x >= self.x && x < self.x + self.width;
        let contains_y = y >= self.y && y < self.y + self.height;

        contains_x && contains_y
    }

    pub fn distance_squared(&self, x: i32, y: i32) -> i32 {
        let dx = [self.x - x, 0, x - (self.x + self.width)]
            .into_iter()
            .max()
            .unwrap();
        let dy = [self.y - y, 0, y - (self.y + self.height)]
            .into_iter()
            .max()
            .unwrap();

        dx * dx + dy * dy
    }

    pub fn fill(&self, draw_context: &mut DrawContext, color: Color) {
        draw_context.rectfill(
            self.x,
            self.y,
            self.x + self.width - 1,
            self.y + self.height - 1,
            color,
        )
    }

    pub fn bottom(&self) -> i32 {
        self.y + self.height - 1
    }

    pub fn right(&self) -> i32 {
        self.x + self.width - 1
    }

    pub fn highlight(
        &self,
        draw_context: &mut DrawContext,
        include_inner: bool,
        highlight_color: Color,
    ) {
        let Rect {
            x,
            y,
            width,
            height,
        } = *self;

        if include_inner {
            draw_context.rect(x, y, x + width - 1, y + height - 1, 0)
        };
        draw_context.rect(x - 1, y - 1, x + width, y + height, highlight_color);
    }
}

fn color_position(color: Color) -> Rect {
    let x = CANVAS_X + 1 + (color as i32 % 4) * SIZE;
    let y = CANVAS_Y + 1 + (color as i32 / 4) * SIZE;

    Rect {
        x,
        y,
        width: SIZE,
        height: SIZE,
    }
}

//// DEBUG STUFF

#[allow(dead_code)]
fn print_debug_strings(draw_context: &mut DrawContext, x: i32, y: i32) {
    draw_context.print(" !\"#$%&'()*+,-.", x, y - 10, 7);
    draw_context.print("0123456789:;<=>?@", x, y, 7);
    let mut letters = "abcdefghijklmnopqrstuvwxyz".to_owned();
    letters.make_ascii_uppercase();
    draw_context.print(&letters, x, y + 10, 7);
}
