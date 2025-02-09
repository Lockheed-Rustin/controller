use eframe::egui::{
    epaint::TextShape, Color32, FontFamily, FontId, Pos2, Shape, Stroke, TextureId, Vec2,
};
use eframe::epaint::{Rect, RectShape, Rounding};
use egui_graphs::{DisplayNode, NodeProps};
use petgraph::{stable_graph::IndexType, EdgeType};

use wg_2024::network::NodeId;
use wg_2024::packet::NodeType;

const RADIUS: f32 = 5.0;
const COLOR: Color32 = Color32::WHITE;

#[derive(Clone)]
pub struct CustomNodeShape {
    label: String,
    node_type: NodeType,
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
            node_type: node_props.payload.1,
            loc: node_props.location(),
        }
    }
}

impl<E: Clone, Ty: EdgeType, Ix: IndexType> DisplayNode<(NodeId, NodeType), E, Ty, Ix>
    for CustomNodeShape
{
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
                FontId::new(ctx.meta.canvas_to_screen_size(5.0), FontFamily::Monospace),
                COLOR,
            )
        });

        let label_offset = Vec2::new(
            -galley.size().x / 2.0,
            -galley.size().y / 2.0 - radius * 2.0,
        );

        // create the shapes
        let mut res = match self.node_type {
            NodeType::Client => CustomNodeShape::get_client_shapes(center, radius),
            NodeType::Drone => CustomNodeShape::get_drone_shapes(center, radius),
            NodeType::Server => CustomNodeShape::get_server_shapes(center, radius),
        };
        let shape_label = TextShape::new(center + label_offset, galley, COLOR);
        res.push(Shape::from(shape_label));
        res
    }

    fn update(&mut self, state: &NodeProps<(NodeId, NodeType)>) {
        self.loc = state.location();
    }

    fn is_inside(&self, pos: Pos2) -> bool {
        pos.distance(self.loc) < RADIUS * 1.3
    }
}

impl CustomNodeShape {
    fn get_client_shapes(screen_center: Pos2, screen_radius: f32) -> Vec<Shape> {
        let shape_circle = Shape::circle_filled(screen_center, screen_radius, COLOR);
        vec![shape_circle]
    }

    fn get_server_shapes(screen_center: Pos2, screen_radius: f32) -> Vec<Shape> {
        let shape_rect = Shape::Rect(RectShape {
            rect: Rect::from_center_size(
                screen_center,
                Vec2::new(screen_radius * 2.0, screen_radius * 2.0),
            ),
            rounding: Rounding::same(screen_radius * 0.2),
            fill: COLOR,
            stroke: Stroke::default(),
            blur_width: 0.0,
            fill_texture_id: TextureId::default(),
            uv: Rect::ZERO,
        });
        vec![shape_rect]
    }

    fn get_drone_shapes(screen_center: Pos2, screen_radius: f32) -> Vec<Shape> {
        let shape_rect = Shape::Rect(RectShape {
            rect: Rect::from_center_size(
                screen_center,
                Vec2::new(screen_radius * 1.2, screen_radius * 1.2),
            ),
            rounding: Rounding::default(),
            fill: COLOR,
            stroke: Stroke::default(),
            blur_width: 0.0,
            fill_texture_id: TextureId::default(),
            uv: Rect::ZERO,
        });
        let mut res = vec![shape_rect];
        for i in [-0.6, 0.6] {
            for j in [-0.6, 0.6] {
                let offset = Vec2::new(screen_radius * i, screen_radius * j);
                let shape_circle =
                    Shape::circle_filled(screen_center + offset, screen_radius * 0.4, COLOR);
                res.push(shape_circle);
            }
        }
        res
    }
}
