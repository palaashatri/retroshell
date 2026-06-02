use crate::{Color, Renderer, Surface};

pub enum RenderNode {
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
        corner_radius: f32,
    },
    Text {
        x: f32,
        y: f32,
        text: String,
        font_size: f32,
        color: Color,
    },
    Image {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        texture_id: u32,
    },
    Group {
        children: Vec<RenderNode>,
    },
}

pub struct RenderTree {
    pub root: RenderNode,
}

impl RenderTree {
    pub fn new(root: RenderNode) -> Self {
        Self { root }
    }

    pub fn draw(&self, renderer: &mut Renderer, surface: &mut Surface) {
        Self::draw_node(&self.root, renderer, surface);
    }

    fn draw_node(node: &RenderNode, _renderer: &mut Renderer, _surface: &mut Surface) {
        match node {
            RenderNode::Rect { .. } => {
                // Rect drawing logic placeholder (structurally complete for GPU batching)
            }
            RenderNode::Text { .. } => {
                // Text drawing logic placeholder
            }
            RenderNode::Image { .. } => {
                // Image drawing logic placeholder
            }
            RenderNode::Group { children } => {
                for child in children {
                    Self::draw_node(child, _renderer, _surface);
                }
            }
        }
    }
}
