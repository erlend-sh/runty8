use crate::app::{AppCompat, ElmApp};
use crate::editor::Editor;
use crate::graphics::{whole_screen_vertex_buffer, FRAGMENT_SHADER, VERTEX_SHADER};
use crate::runtime::draw_context::{DrawContext, DrawData};
use crate::runtime::flags::Flags;
use crate::runtime::map::Map;
use crate::runtime::sprite_sheet::SpriteSheet;
use crate::runtime::state::{InternalState, Scene};
use crate::ui::DispatchEvent;
use crate::{Event, KeyState, MouseButton, MouseEvent};
use crate::{Key, KeyboardEvent, State};
use glium::backend::Facade;
use glium::glutin::dpi::{LogicalPosition, LogicalSize};
use glium::glutin::event::{self, ElementState, KeyboardInput, VirtualKeyCode};
use glium::glutin::event_loop::ControlFlow;
use glium::index::NoIndices;
use glium::texture::{RawImage2d, SrgbTexture2d};
use glium::uniforms::{MagnifySamplerFilter, Sampler};
use glium::{glutin, Program, Surface};
use glium::{uniform, Frame};

pub fn run_app<T: AppCompat + 'static>(
    assets_path: String,
    map: Map,
    sprite_flags: Flags,
    sprite_sheet: SpriteSheet,
    mut draw_data: DrawData,
) {
    let mut internal_state = InternalState::new();
    let mut resources = Resources {
        assets_path,
        sprite_flags,
        sprite_sheet,
        map,
    };
    let state = State::new(&internal_state, &mut resources);
    let mut app = T::init(&state);
    let mut editor = <Editor as ElmApp>::init();

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_inner_size(LogicalSize::new(640.0, 640.0));
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    {
        display.gl_window().window().set_cursor_visible(false);
    }
    let scale_factor = display.gl_window().window().scale_factor();
    let mut logical_size = display
        .gl_window()
        .window()
        .inner_size()
        .to_logical(scale_factor);

    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let program =
        glium::Program::from_source(&display, VERTEX_SHADER, FRAGMENT_SHADER, None).unwrap();

    let mut keys = Keys::new();

    event_loop.run(move |glutin_event, _, control_flow| {
        let event: Option<Event> = handle_event(
            &glutin_event,
            scale_factor,
            &mut logical_size,
            control_flow,
            &mut internal_state,
            &mut keys,
        );

        if let Some(Event::Tick { .. }) = event {
            internal_state.update_keys(&keys);

            if internal_state.escape.btnp() {
                internal_state.scene.flip();
            }
        }

        update_app(
            &mut app,
            &mut editor,
            &mut internal_state,
            &mut resources,
            &mut draw_data,
            event,
        );

        do_draw(
            &display,
            display.draw(),
            &draw_data.buffer,
            &indices,
            &program,
        );
    });
}

fn handle_event(
    event: &event::Event<()>,
    hidpi_factor: f64,
    window_size: &mut LogicalSize<f64>,
    control_flow: &mut ControlFlow,
    state: &mut InternalState,
    keys: &mut Keys,
) -> Option<Event> {
    match event {
        event::Event::WindowEvent { event, .. } => match event {
            glutin::event::WindowEvent::CloseRequested => {
                *control_flow = glutin::event_loop::ControlFlow::Exit;

                None
            }
            // TODO: Force aspect ratio on resize.
            &glutin::event::WindowEvent::Resized(new_size) => {
                let new_size: LogicalSize<f64> = new_size.to_logical(hidpi_factor);

                *window_size = new_size;

                None
            }
            glutin::event::WindowEvent::CursorMoved { position, .. } => {
                let logical_mouse: LogicalPosition<f64> = position.to_logical(hidpi_factor);

                state.mouse_x = (logical_mouse.x / window_size.width * 128.).floor() as i32;
                state.mouse_y = (logical_mouse.y / window_size.height * 128.).floor() as i32;

                Some(Event::Mouse(MouseEvent::Move {
                    x: state.mouse_x,
                    y: state.mouse_y,
                }))
            }
            glutin::event::WindowEvent::MouseInput {
                button: event::MouseButton::Left,
                state: input_state,
                ..
            } => {
                keys.mouse = Some(input_state == &ElementState::Pressed);

                let mouse_event = match input_state {
                    ElementState::Pressed => MouseEvent::Down(MouseButton::Left),
                    ElementState::Released => MouseEvent::Up(MouseButton::Left),
                };

                Some(Event::Mouse(mouse_event))
            }
            glutin::event::WindowEvent::KeyboardInput { input, .. } => {
                handle_key(input, keys).map(Event::Keyboard)
            }
            _ => None,
        },
        event::Event::NewEvents(cause) => match cause {
            glutin::event::StartCause::ResumeTimeReached {
                start,
                requested_resume,
            } => {
                set_next_timer(control_flow);

                let delta: Result<i32, _> = requested_resume
                    .duration_since(*start)
                    .as_millis()
                    .try_into();

                Some(Event::Tick {
                    delta_millis: delta.unwrap().try_into().unwrap(),
                })
            }
            glutin::event::StartCause::Init => {
                set_next_timer(control_flow);

                None
            }
            _ => None,
        },
        _ => None,
    }
}

fn handle_key(input: &KeyboardInput, keys: &mut Keys) -> Option<KeyboardEvent> {
    let key = input.virtual_keycode?;

    let mut other = None;
    let key_ref = match key {
        VirtualKeyCode::X => &mut keys.x,
        VirtualKeyCode::C => &mut keys.c,
        VirtualKeyCode::Left => &mut keys.left,
        VirtualKeyCode::Up => &mut keys.up,
        VirtualKeyCode::Right => &mut keys.right,
        VirtualKeyCode::Down => &mut keys.down,
        VirtualKeyCode::Escape => &mut keys.escape,
        _ => &mut other,
    };
    *key_ref = Some(input.state == ElementState::Pressed);

    let runty8_key = Key::from_virtual_keycode(key)?;
    let state = KeyState::from_state(input.state);

    Some(KeyboardEvent {
        key: runty8_key,
        state,
    })
}

pub(crate) struct Keys {
    pub(crate) left: Option<bool>,
    pub(crate) right: Option<bool>,
    pub(crate) up: Option<bool>,
    pub(crate) down: Option<bool>,
    pub(crate) x: Option<bool>,
    pub(crate) c: Option<bool>,
    pub(crate) escape: Option<bool>,
    pub(crate) mouse: Option<bool>,
}

impl Keys {
    pub(crate) fn new() -> Self {
        Self {
            left: None,
            right: None,
            up: None,
            down: None,
            x: None,
            c: None,
            escape: None,
            mouse: None,
        }
    }
}

pub struct Resources {
    pub assets_path: String,
    pub sprite_sheet: SpriteSheet,
    pub sprite_flags: Flags,
    pub map: Map,
}

fn update_app<'a>(
    app: &'a mut (impl AppCompat + 'static),
    editor: &'a mut Editor,
    internal_state: &'a mut InternalState,
    resources: &'a mut Resources,
    draw_data: &'a mut DrawData,
    event: Option<Event>,
) {
    match internal_state.scene {
        Scene::App => refresh_app(app, resources, internal_state, draw_data, event),
        Scene::Editor => refresh_app(editor, resources, internal_state, draw_data, event),
    }
}

fn refresh_app(
    app: &mut impl AppCompat,
    resources: &mut Resources,
    internal_state: &mut InternalState,
    draw_data: &mut DrawData,
    event: Option<Event>,
) {
    let mut view = app.view(resources, internal_state, draw_data);

    let mut msg_queue = vec![];
    let dispatch_event = &mut DispatchEvent::new(&mut msg_queue);

    if let Some(event) = event {
        view.as_widget_mut().on_event(
            event,
            (internal_state.mouse_x, internal_state.mouse_y),
            dispatch_event,
        );
    }

    let mut state = State::new(internal_state, resources);
    let mut draw_context = DrawContext::new(&mut state, draw_data);
    view.as_widget().draw(&mut draw_context);
    drop(view);

    if let Some(sub_msg) = event.and_then(|e| app.subscriptions(&e)) {
        msg_queue.push(sub_msg);
    }
    for msg in msg_queue.into_iter() {
        app.update(&msg, resources, internal_state);
    }
}

fn set_next_timer(control_flow: &mut ControlFlow) {
    let fps = 30_u64;
    let nanoseconds_per_frame = 1_000_000_000 / fps;

    let next_frame_time =
        std::time::Instant::now() + std::time::Duration::from_nanos(nanoseconds_per_frame);
    *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
}

fn do_draw(
    display: &impl Facade,
    mut target: Frame,
    buffer: &[u8],
    indices: &NoIndices,
    program: &Program,
) {
    target.clear_color(1.0, 0.0, 0.0, 1.0);
    let image = RawImage2d::from_raw_rgb(buffer.to_vec(), (128, 128));
    let texture = SrgbTexture2d::new(display, image).unwrap();
    let uniforms = uniform! {
        tex: Sampler::new(&texture).magnify_filter(MagnifySamplerFilter::Nearest)
    };
    target
        .draw(
            &whole_screen_vertex_buffer(display),
            indices,
            program,
            &uniforms,
            &Default::default(),
        )
        .unwrap();
    target.finish().unwrap();
}
