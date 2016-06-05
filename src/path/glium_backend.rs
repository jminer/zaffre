
use std::any::Any;
use std::rc::Rc;
use std::sync::Arc;
use glium::{self, Surface};
use glium::backend::Context;
use glium::framebuffer::{RenderBuffer, SimpleFrameBuffer};
use glium::texture::UncompressedFloatFormat;
use glium::vertex::VertexBuffer;
use glium::index::{IndexBuffer, PrimitiveType};
use super::{PathBuf, SolidVertex};

implement_vertex!(SolidVertex, position);

const STROKE_SOLID_VERT_SHADER: &'static str = include_str!("../shaders/stroke_solid.vert");
const STROKE_SOLID_FRAG_SHADER: &'static str = include_str!("../shaders/stroke_solid.frag");

struct GliumBufferCache<T> where T: Copy {
    vertex_buffer: VertexBuffer<T>,
    index_buffer: IndexBuffer<u16>,
}

struct GliumStrokeCache {
    solid: GliumBufferCache<SolidVertex>,
    //quad_bezier: GliumBufferCache<StrokeQuadBezierVertex>,
}

fn build_stroke_cache<F>(facade: &F, path: &mut PathBuf) -> Arc<GliumStrokeCache>
                         where F: glium::backend::Facade {
    path.bake_stroke();

    let context: &Context = &**facade.get_context();
    // unwrap() can't panic because of bake_stroke()
    let mut baked_stroke = path.baked_stroke.as_mut().unwrap();
    let solid_geo = &baked_stroke.solid_geo;
    let cache = baked_stroke.backend.entry(context as *const _ as usize).or_insert_with(|| {
        Arc::new(GliumStrokeCache {
            solid: GliumBufferCache {
                vertex_buffer: VertexBuffer::new(facade, &solid_geo.vertices).unwrap(),
                index_buffer: IndexBuffer::new(facade,
                                               PrimitiveType::TrianglesList,
                                               &solid_geo.indices).unwrap(),
            },
        }) as Arc<Any>
    });
    cache.downcast_ref::<Arc<GliumStrokeCache>>().unwrap().clone()
}

pub fn stencil_stroke_path<F, S>(facade: &F, mut surface: S, path: &mut PathBuf)
                                 where F: glium::backend::Facade,
                                       S: Surface {
    assert!(surface.has_stencil_buffer());

    let cache = build_stroke_cache(facade, path);

    // TODO: remove unwraps from this function
    // painting code should never panic...
    let solid_program = glium::program::Program::from_source(facade,
                                                             STROKE_SOLID_VERT_SHADER,
                                                             STROKE_SOLID_FRAG_SHADER,
                                                             None).unwrap();

    let draw_params = glium::draw_parameters::DrawParameters {
        color_mask: (false, false, false, false),
        .. Default::default()
    };
    surface.draw(&cache.solid.vertex_buffer, &cache.solid.index_buffer, &solid_program,
    &glium::uniforms::EmptyUniforms, &draw_params).unwrap();
}

// not sure covering is necessary
// pub fn cover<T: glium::Surface>(surface: &T, path: &mut PathBuf) {
// }
