use eframe::egui::{Color32, Pos2, Shape, Stroke};
use egui_graphs::{DefaultEdgeShape, DisplayEdge, DisplayNode, DrawContext, EdgeProps, Node};
use petgraph::{stable_graph::IndexType, EdgeType};

const COLOR: Color32 = Color32::from_rgb(70, 70, 70);
const WIDTH: f32 = 3.0;

/// Struct for rendering a custom edge in the Topology section
#[derive(Clone)]
pub struct EdgeShape {
    default_impl: DefaultEdgeShape,
}

impl<E: Clone> From<EdgeProps<E>> for EdgeShape {
    fn from(props: EdgeProps<E>) -> Self {
        Self {
            default_impl: DefaultEdgeShape::from(props),
        }
    }
}

impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType, D: DisplayNode<N, E, Ty, Ix>>
    DisplayEdge<N, E, Ty, Ix, D> for EdgeShape
{
    fn shapes(
        &mut self,
        start: &Node<N, E, Ty, Ix, D>,
        end: &Node<N, E, Ty, Ix, D>,
        ctx: &DrawContext,
    ) -> Vec<Shape> {
        let mut res = vec![];
        let (start, end) = (start.location(), end.location());

        let mut stroke = Stroke::new(WIDTH, COLOR);

        stroke.width = ctx.meta.canvas_to_screen_size(stroke.width);
        res.push(Shape::line_segment(
            [
                ctx.meta.canvas_to_screen_pos(start),
                ctx.meta.canvas_to_screen_pos(end),
            ],
            stroke,
        ));

        res
    }

    fn update(&mut self, _: &EdgeProps<E>) {}

    fn is_inside(
        &self,
        start: &Node<N, E, Ty, Ix, D>,
        end: &Node<N, E, Ty, Ix, D>,
        pos: Pos2,
    ) -> bool {
        self.default_impl.is_inside(start, end, pos)
    }
}
