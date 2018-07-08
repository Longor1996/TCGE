extern crate failure;
extern crate TCGE;
extern crate time;

extern crate glfw;
use self::glfw::{Context, Key, Action};

extern crate gl;

use TCGE::resources::Resources;
use TCGE::client::render_gl;

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

    // ------------------------------------------
    let (mut window, events) = glfw.create_window(
        1024, 768, "Talecraft",
        glfw::WindowMode::Windowed
    ).expect("Failed to create GLFW window.");

    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    // ------------------------------------------
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    // ------------------------------------------
    let shader_program = render_gl::Program::from_res(
        &res, "shaders/triangle"
    )?;

    shader_program.set_used();

    // ------------------------------------------

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


    // ------------------------------------------
    while !window.should_close() {
        process_events(&mut window, &events);
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        shader_program.set_used();
        unsafe {
            gl::BindVertexArray(vao);
            gl::DrawArrays(
                gl::TRIANGLES,
                0, (vertices.len()/3) as i32
            );
        }

        window.swap_buffers();
        glfw.poll_events();
    }

    Ok(())
}

use std::sync::mpsc::Receiver;
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

pub fn failure_to_string<E: failure::Fail>(e: E) -> String {
    use std::fmt::Write;

    let mut result = String::new();

    for (i, cause) in e.causes().collect::<Vec<_>>().into_iter().rev().enumerate() {
        if i > 0 {
            let _ = writeln!(&mut result, "   Which caused the following issue:");
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

    result
}
