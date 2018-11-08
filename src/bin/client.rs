use std::rc::Rc;
use std::cell::Ref;
use std::cell::RefMut;
use std::cell::RefCell;

extern crate failure;
use failure::Fail;

extern crate time;
extern crate cgmath;

extern crate glfw;
use self::glfw::{Context, Key, Action};

extern crate gl;

extern crate TCGE;
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
    window.set_cursor_pos_polling(true);
    window.set_cursor_mode(glfw::CursorMode::Disabled);
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
    let render_state = RenderState {
        uniform_mat: shader_program.uniform_location("transform"),
        uniform_time: shader_program.uniform_location("time"),
        shader_program: shader_program
    };

    let mut cursor = Cursor {pos_x: 0.0, pos_y: 0.0, mov_x: 0.0, mov_y: 0.0};

    let scene = Rc::new(RefCell::new(Option::Some(Scene {
        camera: Camera {
            position: cgmath::Vector3 {x: 0.0, y: 1.8, z: 0.0},
            velocity: cgmath::Vector3 {x: 0.0, y: 0.0, z: 0.0},
            rotation: cgmath::Vector2 {x: 0.0, y: 90.0}
        },
        meshes: vec![geometry_test()]
    })));

    // ------------------------------------------
    let mut gls = gameloop::newGameloop(20);

    while !window.should_close() {
        process_events(
            &mut window,
            &events,
            &mut cursor,
            &mut *scene.borrow_mut()
        );

        let window_size = window.get_framebuffer_size();

        gameloop::gameloop_next(&mut gls,
            || {glfw.get_time()},

            |now:f64| {
                // println!("It is now {}", now);

                scene.borrow().as_ref().map(|scene| {
                    println!("CAMERA {}", scene.camera);
                });

                scene.borrow_mut().as_mut().map(|mut_scene| {
                    mut_scene.camera.update_movement(&window);
                });
            },

            |now:f64, interpolation:f32| {
                scene.borrow().as_ref().map(|scene| {
                    render(
                        &render_state,
                        &scene,
                        &scene.camera,
                        window_size,
                        now,
                        interpolation
                    )
                });

            }
        );

        window.swap_buffers();
        glfw.poll_events();
    }

    Ok(())
}

struct Scene {
    camera: Camera,
    meshes: Vec<SimpleVAO>,
}

struct Cursor {
    pos_x: f32,
    pos_y: f32,
    mov_x: f32,
    mov_y: f32,
}

impl Cursor {
    fn update(&mut self, x: f64, y: f64) {
        self.mov_x = (x as f32) - self.pos_x;
        self.mov_y = (y as f32) - self.pos_y;
        self.pos_x = x as f32;
        self.pos_y = y as f32;
    }
}

struct RenderState {
    uniform_mat: i32,
    uniform_time: i32,
    shader_program: render_gl::Program
}

fn render(render_state: &RenderState, scene: &Scene, camera: &Camera, size: (i32, i32), now: f64, interpolation:f32) {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        gl::Enable(gl::DEPTH_TEST);
        gl::CullFace(gl::FRONT);
        gl::Enable(gl::CULL_FACE);
    }

    render_state.shader_program.set_used();
    render_state.shader_program.uniform_matrix4(render_state.uniform_mat, camera.transform(size));
    render_state.shader_program.uniform_scalar(render_state.uniform_time, now as f32);

    for mesh in scene.meshes.iter() {
        unsafe {
            gl::BindVertexArray(mesh.handle);
            gl::DrawArrays(
                gl::TRIANGLES,
                0, mesh.count
            );
        }
    }
}

use std::sync::mpsc::Receiver;
fn process_events(
    window: &mut glfw::Window,
    events: &Receiver<(f64, glfw::WindowEvent)>,
    cursor: &mut Cursor,
    opt_scene: &mut Option<Scene>
) {
    for(_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                unsafe {gl::Viewport(0, 0, width, height)}
            },
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            },
            glfw::WindowEvent::CursorPos(x, y) => {
                cursor.update(x, y);
                opt_scene.as_mut()
                    .map(|mut_scene| &mut mut_scene.camera)
                    .map( |mut_camera| {
                        mut_camera.update_rotation(
                            cursor.mov_x,
                            cursor.mov_y
                        );
                    });
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
    let mut vertices: Vec<f32> = vec![
        -0.5, -0.5, -10.0,
        0.5, -0.5, -10.0,
        0.0, 0.5, -10.0
    ];

    vertices.extend(&vec![
        -20.0, 0.0, -20.0,
          0.0, 0.0,  20.0,
         20.0, 0.0, -20.0
    ]);

    vertices.extend(&vec![
        -5.0, 0.0, 30.0,
         0.0, 9.0, 30.0,
         5.0, 0.0, 30.0
    ]);

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

// TODO: Camera needs PlayerController/ClientInput...
#[derive(Debug)]
struct Camera {
    position: cgmath::Vector3<f32>,
    velocity: cgmath::Vector3<f32>,
    rotation: cgmath::Vector2<f32>,
}

impl Camera {
    fn transform(&self, size: (i32,i32) ) -> cgmath::Matrix4<f32> {
        use cgmath::Matrix4;

        let (width, height) = size;
        let fov = cgmath::Rad::from(cgmath::Deg(90.0));

        let perspective = cgmath::PerspectiveFov {
            fovy: fov,
            aspect: width as f32 / height as f32,
            near: 0.1, far: 1024.0
        };

        let perspective = Matrix4::from(perspective);

        // this next section can most certainly be written with less code...
        let mut camera = Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0
        );

        let pitch = cgmath::Deg(self.rotation.x);
        let yaw = cgmath::Deg(self.rotation.y);

        camera = camera * Matrix4::from_angle_x(pitch);
        camera = camera * Matrix4::from_angle_y(yaw);
        camera = camera * Matrix4::from_nonuniform_scale(1.0,1.0,-1.0);
        camera = camera * Matrix4::from_translation(-self.position);

        return perspective * camera;
    }

    fn update_rotation(&mut self, yaw: f32, pitch: f32) {
        let mouse_sensivity = 0.5;

        self.rotation.x += pitch * mouse_sensivity;
        if self.rotation.x < -90.0 {
            self.rotation.x = -90.0;
        }
        if self.rotation.x > 90.0 {
            self.rotation.x = 90.0;
        }

        self.rotation.y += yaw * mouse_sensivity;
        while self.rotation.y < 0.0 {
            self.rotation.y += 360.0;
        }
        while self.rotation.y > 360.0 {
            self.rotation.y -= 360.0;
        }
    }

    fn update_movement(&mut self, window: & glfw::Window) {
        use cgmath::Vector3;
        use cgmath::Matrix4;
        use cgmath::Transform;

        let move_speed = 0.5;

        if window.get_key(Key::LeftShift) == Action::Press {
            self.position += Vector3::new(0.0, -1.0, 0.0) * move_speed;
        }
        if window.get_key(Key::Space) == Action::Press {
            self.position += Vector3::new(0.0, 1.0, 0.0) * move_speed;
        }

        let yaw = cgmath::Deg(self.rotation.y);
        let mat = Matrix4::from_angle_y(yaw);

        let forward = Vector3::new(0.0, 0.0, 1.0);
        let forward = Matrix4::transform_vector(&mat, forward);
        if window.get_key(Key::W) == Action::Press {
            self.position += forward * move_speed;
        }

        let backward = Vector3::new(0.0, 0.0, -1.0);
        let backward = Matrix4::transform_vector(&mat, backward);
        if window.get_key(Key::S) == Action::Press {
            self.position += backward * move_speed;
        }

        let left = Vector3::new(-1.0, 0.0, 0.0);
        let left = Matrix4::transform_vector(&mat, left);
        if window.get_key(Key::A) == Action::Press {
            self.position += left * move_speed;
        }

        let right = Vector3::new(1.0, 0.0, 0.0);
        let right = Matrix4::transform_vector(&mat, right);
        if window.get_key(Key::D) == Action::Press {
            self.position += right * move_speed;
        }
    }
}

impl std::fmt::Display for Camera {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "Camera [x: {}, z: {}, pitch: {}, yaw: {} ]",
               self.position.x,
               self.position.z,
               self.rotation.x,
               self.rotation.y
        )
    }
}