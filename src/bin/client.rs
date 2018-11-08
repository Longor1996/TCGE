extern crate failure;
extern crate TCGE;
extern crate time;
extern crate cgmath;
extern crate glfw;
use self::glfw::{Context, Key, Action};

extern crate gl;

use TCGE::resources::Resources;
use TCGE::client::render_gl;
use TCGE::gameloop;

fn main() {
    println!("Hello, Client!");

    if let Err(e) = run() {
        use std::fmt::Write;
        let mut result = String::new();

        for (i, cause) in e.causes().collect::<Vec<_>>().into_iter().enumerate() {
            if i > 0 {
                let _ = write!(&mut result, "   Caused by: ");
            }
            let _ = write!(&mut result, "{}", cause);
            if let Some(backtrace) = cause.backtrace() {
                let backtrace_str = format!("{}", backtrace);
                if backtrace_str.len() > 0 {
                    let _ = writeln!(&mut result, " This happened at {}", backtrace);
                } else {
                    let _ = writeln!(&mut result);
                }
            } else {
                let _ = writeln!(&mut result);
            }
        }

        println!("{}", result);
    }

    println!("Goodbye!");
}

fn run() -> Result<(), failure::Error> {
    // ------------------------------------------
    let res = Resources::from_exe_path()?;

    // ------------------------------------------
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;

    glfw.window_hint(glfw::WindowHint::ContextVersion(3,2));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));

    // ------------------------------------------
    let (mut window, events) = glfw.create_window(
        1024, 768, "Talecraft",
        glfw::WindowMode::Windowed
    ).expect("Failed to create GLFW window.");

    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_size_limits(
        320, 225,
        glfw::ffi::DONT_CARE as u32,
        glfw::ffi::DONT_CARE as u32
    );

    // ------------------------------------------
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    // ------------------------------------------
    let shader_program = render_gl::Program::from_res(
        &res, "shaders/triangle"
    )?;

    shader_program.set_used();

    // ------------------------------------------
    let geometry = geometry_test();

    let mut render_state = RenderState {
        uniform_mat: shader_program.uniform_location("transform"),
        uniform_time: shader_program.uniform_location("time"),
        shader_program: shader_program,
        geometry: geometry
    };

    let camera = Camera {
        position: cgmath::Vector3 {x: 0.0, y: 0.0, z: 0.0},
        velocity: cgmath::Vector3 {x: 0.0, y: 0.0, z: 0.0},
        rotation: cgmath::Vector2 {x: 0.0, y: 0.0}
    };

    let mut gls = gameloop::newGameloop(20);

    // ------------------------------------------
    while !window.should_close() {
        process_events(&mut window, &events);

        gameloop::gameloop_next(&mut gls,
            || {glfw.get_time()},
            |now:f64| {
                println!("It is now {}", now);
            },
            |now:f64, interpolation:f32| {
                render(&render_state, &camera, now);
            }
        );
        
        window.swap_buffers();
        glfw.poll_events();
    }

    Ok(())
}

struct RenderState {
    uniform_mat: i32,
    uniform_time: i32,
    shader_program: render_gl::Program,
    geometry: SimpleVAO,
}

fn render(render_state: &RenderState, camera: &Camera, now: f64) {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    render_state.shader_program.set_used();
    render_state.shader_program.uniform_matrix4(render_state.uniform_mat, camera.transform());
    render_state.shader_program.uniform_scalar(render_state.uniform_time, now as f32);
    unsafe {
        gl::BindVertexArray(render_state.geometry.handle);
        gl::DrawArrays(
            gl::TRIANGLES,
            0, render_state.geometry.count
        );
    }
}

use std::sync::mpsc::Receiver;
use failure::Fail;

fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    for(_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                unsafe {gl::Viewport(0, 0, width, height)}
            },
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            },
            _ => ()
        }
    }
}

struct SimpleVAO {
    handle: gl::types::GLuint,
    count: i32,
}

fn geometry_test() -> SimpleVAO {
    let vertices: Vec<f32> = vec![
        -0.5, -0.5, 0.0,
        0.5, -0.5, 0.0,
        0.0, 0.5, 0.0
    ];

    let mut vbo: gl::types::GLuint = 0;
    unsafe {
        gl::GenBuffers(1, &mut vbo);
    }
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            vertices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    let mut vao: gl::types::GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT, gl::FALSE,
            (3 * std::mem::size_of::<f32>()) as gl::types::GLint,
            std::ptr::null()
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    return SimpleVAO {
        handle: vao,
        count: (vertices.len()/3) as i32
    }
}

struct Camera {
    position: cgmath::Vector3<f32>,
    velocity: cgmath::Vector3<f32>,
    rotation: cgmath::Vector2<f32>,
}

impl Camera {
    fn transform(&self) -> cgmath::Matrix4<f32> {
        let mat = cgmath::Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0
        );

        return mat
    }
}