
#[macro_use]
extern crate glium;

extern crate nalgebra;
extern crate num;
extern crate zaffre;

use glium::{DisplayBuild, Surface};
use glium::glutin;
use nalgebra::{Matrix2, Matrix4, Point2, ToHomogeneous, Transpose};
use num::One;

#[derive(Copy, Clone, PartialEq)]
struct Vertex {
    position: (f32, f32, f32),
}

implement_vertex!(Vertex, position);

static VERT_SHADER: &'static str = include_str!("basic_shader.vert");
static FRAG_SHADER: &'static str = include_str!("basic_shader.frag");

fn main() {
    let display = glutin::WindowBuilder::new()
                  .with_dimensions(850, 480)
                  .with_title(String::from("Test"))
                  .with_multisampling(8)
                  //.with_vsync()
                  .build_glium()
                  .unwrap();
    let window = display.get_window().unwrap();

    let mut path = zaffre::PathBuf::new();
    path.set_stroke_width(20.0);
    // path.move_to(Point2::new(20.0, 10.0));
    // path.line_to(Point2::new(40.0, 15.0));
    path.move_to(Point2::new(10.0, 20.0));
    path.line_to(Point2::new(80.0, 30.0));
    path.line_to(Point2::new(40.0, 50.0));
    path.quad_curve_to(Point2::new(20.0, 100.0), Point2::new(100.0, 200.0));

    let vertices = &[
        Vertex { position: (-1.0,  1.0, 0.0) },
        Vertex { position: ( 1.0,  1.0, 0.0) },
        Vertex { position: ( 1.0, -1.0, 0.0) },
        Vertex { position: (-1.0, -1.0, 0.0) },
    ];
    let vertex_buffer = glium::vertex::VertexBuffer::new(&display, vertices).unwrap();
    let indices: &[u16] = &[0, 3, 1, 1, 3, 2];
    let index_buffer = glium::index::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, indices).unwrap();

    let program = glium::program::Program::from_source(&display, &VERT_SHADER, &FRAG_SHADER, None).unwrap();

    let mut closed = false;
    while !closed {
        let (win_width, win_height) = window.get_inner_size_points()
                                            .expect("failed getting window size");

        let mut transform = Matrix4::one();
        // scale -1.0 .. 1.0 to -window_width/2 .. window_width/2
        // The starting range is 2.0 and the ending range is win_width.
        // Also, flip Y.
        transform.m11 = 2.0 / (win_width as f32);
        transform.m22 = -2.0 / (win_height as f32);
        // Translate (0, 0) to the top left of the screen.
        transform.m41 = -1.0;
        transform.m42 = 1.0;
        // println!("transf: {:#?}", transform);
        // println!("pt: {:#?}", nalgebra::Point4::new(20.0f32, 30.0, 0.0, 1.0));

        let mut target = display.draw();
        target.clear_stencil(0);

        zaffre::stencil_stroke_path(&display, &mut target, &mut path, &transform);

        let uniforms = uniform! {
        };
        let draw_params = glium::draw_parameters::DrawParameters {
            stencil: glium::draw_parameters::Stencil {
                test_clockwise: glium::StencilTest::IfEqual { mask: 0xFF },
                reference_value_clockwise: 1,
                test_counter_clockwise: glium::StencilTest::IfEqual { mask: 0xFF },
                reference_value_counter_clockwise: 1,
                ..Default::default()
            },
            ..Default::default()
        };
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &draw_params).unwrap();
        target.finish();

        for ev in display.poll_events() {
            match ev {
                glutin::Event::Closed => closed = true,
                glutin::Event::MouseInput(state, button) => {
                },
                glutin::Event::KeyboardInput(state, _, Some(key)) => {
                },
                _ => {}
            }
        }
    }
}
