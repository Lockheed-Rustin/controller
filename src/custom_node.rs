use eframe::egui::{epaint::TextShape, FontFamily, FontId, Pos2, Shape, Vec2};
use egui_graphs::{DisplayNode, NodeProps};
use petgraph::{stable_graph::IndexType, EdgeType};

const RADIUS: f32 = 5.0;

#[derive(Clone)]
pub struct CustomNodeShape {
    label: String,
    loc: Pos2,
}

impl<N: Clone> From<NodeProps<N>> for CustomNodeShape {
    fn from(node_props: NodeProps<N>) -> Self {
        Self {
            label: node_props.label.clone(),
            loc: node_props.location(),
        }
    }
}

impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> DisplayNode<N, E, Ty, Ix>
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
        let color = ctx.ctx.style().visuals.text_color();

        // create label
        let galley = ctx.ctx.fonts(|f| {
            f.layout_no_wrap(
                self.label.clone(),
                FontId::new(ctx.meta.canvas_to_screen_size(7.0), FontFamily::Monospace),
                color,
            )
        });

        let label_offset = Vec2::new(
            -galley.size().x / 2.0,
            -galley.size().y / 2.0 - radius * 2.0,
        );

        // create the shapes
        let shape_label = TextShape::new(center + label_offset, galley, color);
        // let shape_circle = Shape::circle_stroke(center, radius, Stroke::new(2.0, color));
        let shape_circle = Shape::circle_filled(center, radius, color);

        vec![shape_circle, shape_label.into()]
    }

    fn update(&mut self, state: &NodeProps<N>) {
        self.label = state.label.clone();
        self.loc = state.location();
    }
}
