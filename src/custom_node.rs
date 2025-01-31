use eframe::egui::{epaint::TextShape, Color32, FontFamily, FontId, Pos2, Shape, TextBuffer, Vec2};
use egui_graphs::{DisplayNode, NodeProps};
use petgraph::{stable_graph::IndexType, EdgeType};


use wg_2024::network::NodeId;
use crate::app::NodeType;

const RADIUS: f32 = 5.0;
const COLOR: Color32 = Color32::WHITE;

#[derive(Clone)]
pub struct CustomNodeShape {
    label: String,
    loc: Pos2,
}

impl From<NodeProps<(NodeId, NodeType)>> for CustomNodeShape {
    fn from(node_props: NodeProps<(NodeId, NodeType)>) -> Self {
        let mut label = match node_props.payload.1 {
            NodeType::Client => "Client #".to_string(),
            NodeType::Drone => "Drone #".to_string(),
            NodeType::Server => "Server #".to_string(),
        };
        label.push_str(&node_props.payload.0.to_string());
        Self {
            label,
            loc: node_props.location(),
        }
    }
}

impl<E: Clone, Ty: EdgeType, Ix: IndexType> DisplayNode<(NodeId, NodeType), E, Ty, Ix>
    for CustomNodeShape
{
    fn is_inside(&self, pos: Pos2) -> bool {
        pos.distance(self.loc) < RADIUS * 1.3
    }

    fn closest_boundary_point(&self, _dir: Vec2) -> Pos2 {
        self.loc
    }

    fn shapes(&mut self, ctx: &egui_graphs::DrawContext) -> Vec<Shape> {
        // relative to screen
        let center = ctx.meta.canvas_to_screen_pos(self.loc);
        let radius = ctx.meta.canvas_to_screen_size(RADIUS);
        // let color = ctx.ctx.style().visuals.text_color();

        // create label
        let galley = ctx.ctx.fonts(|f| {
            f.layout_no_wrap(
                self.label.clone(),
                FontId::new(ctx.meta.canvas_to_screen_size(7.0), FontFamily::Monospace),
                COLOR,
            )
        });

        let label_offset = Vec2::new(
            -galley.size().x / 2.0,
            -galley.size().y / 2.0 - radius * 2.0,
        );

        // create the shapes
        let shape_label = TextShape::new(center + label_offset, galley, COLOR);
        let shape_circle = Shape::circle_filled(center, radius, COLOR);

        vec![shape_circle, shape_label.into()]
    }

    fn update(&mut self, state: &NodeProps<(NodeId, NodeType)>) {
        self.loc = state.location();
    }
}
