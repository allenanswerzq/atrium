//! GPUI renderer for [`ViewSpec`].
//!
//! Nodes are **GPUI elements** — clickable, hoverable, with real text.
//! Edges and groups are painted on a **canvas** behind the nodes.
//! All visual decisions come from the `ViewSpec`. The renderer is dumb.
//!
//! Interactions: left-click (select/open), double-click (drill-down),
//! right-click (highlight connections), scroll (zoom), drag (pan).

use gpui::{
    canvas, div, point, prelude::*, px, rgb, size, transparent_black, BorderStyle, Bounds,
    Context, Corners, Edges, Hsla, MouseButton, PaintQuad, Pixels, ScrollWheelEvent, Window,
};

use atrium_graph_view::{Action, Edge, Group, Node, NodeShape, ViewSpec};

use super::fake_data;

// ── Panel ───────────────────────────────────────────────────────────

/// GPUI component that renders a [`ViewSpec`].
pub struct GraphViewPanel {
    spec: ViewSpec,
    /// View stack for back navigation.
    view_stack: Vec<ViewSpec>,
    /// Currently selected node ID.
    selected: Option<String>,
    /// Pan offset in logical pixels.
    pan_x: f32,
    pan_y: f32,
    /// Zoom level (1.0 = 100%).
    zoom: f32,
    /// Drag state for panning.
    dragging: bool,
    drag_start_x: f32,
    drag_start_y: f32,
}

impl GraphViewPanel {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            spec: fake_data::atrium_crate_view(),
            view_stack: Vec::new(),
            selected: None,
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
            dragging: false,
            drag_start_x: 0.0,
            drag_start_y: 0.0,
        }
    }

    fn push_view(&mut self, new_spec: ViewSpec) {
        let old = std::mem::replace(&mut self.spec, new_spec);
        self.view_stack.push(old);
        self.selected = None;
        self.pan_x = 0.0;
        self.pan_y = 0.0;
        self.zoom = 1.0;
    }

    fn pop_view(&mut self) {
        if let Some(prev) = self.view_stack.pop() {
            self.spec = prev;
            self.selected = None;
            self.pan_x = 0.0;
            self.pan_y = 0.0;
            self.zoom = 1.0;
        }
    }
}

impl Render for GraphViewPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let spec = &self.spec;
        let zoom = self.zoom;
        let pan_x = self.pan_x;
        let pan_y = self.pan_y;
        let selected = self.selected.clone();

        let groups = spec.groups.clone();
        let edges = spec.edges.clone();
        let selected_edges = selected.clone();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0xFFFFFFu32))
            .child(render_breadcrumb(&spec.breadcrumb, !self.view_stack.is_empty(), cx))
            .child(
                div()
                    .id("graph-canvas")
                    .flex_1()
                    .w_full()
                    .relative()
                    .overflow_hidden()
                    // Scroll to zoom
                    .on_scroll_wheel(cx.listener(|this, ev: &ScrollWheelEvent, _window, cx| {
                        let delta = f32::from(ev.delta.pixel_delta(px(1.0)).y);
                        let factor = if delta > 0.0 { 1.1 } else { 0.9 };
                        this.zoom = (this.zoom * factor).clamp(0.3, 3.0);
                        cx.notify();
                    }))
                    // Drag to pan — start
                    .on_mouse_down(MouseButton::Middle, cx.listener(|this, ev: &gpui::MouseDownEvent, _window, cx| {
                        this.dragging = true;
                        this.drag_start_x = f32::from(ev.position.x) - this.pan_x;
                        this.drag_start_y = f32::from(ev.position.y) - this.pan_y;
                        cx.notify();
                    }))
                    .on_mouse_up(MouseButton::Middle, cx.listener(|this, _ev: &gpui::MouseUpEvent, _window, cx| {
                        this.dragging = false;
                        cx.notify();
                    }))
                    .on_mouse_move(cx.listener(|this, ev: &gpui::MouseMoveEvent, _window, cx| {
                        if this.dragging {
                            this.pan_x = f32::from(ev.position.x) - this.drag_start_x;
                            this.pan_y = f32::from(ev.position.y) - this.drag_start_y;
                            cx.notify();
                        }
                    }))
                    // Click on empty space to deselect
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _ev: &gpui::MouseDownEvent, _window, cx| {
                        this.selected = None;
                        cx.notify();
                    }))
                    // Transform container (zoom + pan)
                    .child(
                        div()
                            .absolute()
                            .left(px(pan_x))
                            .top(px(pan_y))
                            .w(px(2000.0 * zoom))
                            .h(px(1500.0 * zoom))
                            // Canvas: edges + groups
                            .child(
                                canvas(
                                    |_bounds, _window, _cx| {},
                                    {
                                        let sel = selected_edges;
                                        move |_bounds, _state, window, _cx| {
                                            for group in &groups {
                                                draw_group(window, group, zoom);
                                            }
                                            for edge in &edges {
                                                let highlight = sel.as_ref().is_some_and(|s| {
                                                    edge.from == *s || edge.to == *s
                                                });
                                                draw_edge(window, edge, zoom, highlight);
                                            }
                                        }
                                    },
                                )
                                .absolute()
                                .size_full(),
                            )
                            // Nodes
                            .children(spec.nodes.iter().map(|node| {
                                render_node(node, zoom, selected.as_deref(), cx)
                            })),
                    ),
            )
            .child(render_status(&spec.status, self.zoom))
    }
}

// ── Breadcrumb ──────────────────────────────────────────────────────

fn render_breadcrumb(segments: &[String], has_back: bool, cx: &mut Context<GraphViewPanel>) -> gpui::Div {
    let text = if segments.is_empty() {
        "Graph View".to_owned()
    } else {
        segments.join("  >  ")
    };

    let mut bar = div()
        .h(px(40.0))
        .w_full()
        .bg(rgb(0xFAFAFAu32))
        .border_b_1()
        .border_color(rgb(0xE0E0E0u32))
        .flex()
        .items_center()
        .gap(px(8.0))
        .px(px(16.0));

    if has_back {
        bar = bar.child(
            div()
                .px(px(8.0))
                .py(px(4.0))
                .bg(rgb(0xE0E0E0u32))
                .rounded(px(4.0))
                .cursor_pointer()
                .text_size(px(12.0))
                .text_color(rgb(0x333333u32))
                .hover(|s| s.bg(rgb(0xBDBDBDu32)))
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _ev, _window, cx| {
                    this.pop_view();
                    cx.notify();
                }))
                .child("← Back"),
        );
    }

    bar.child(div().text_size(px(14.0)).text_color(rgb(0x222222u32)).child(text))
}

// ── Status bar ──────────────────────────────────────────────────────

fn render_status(status: &str, zoom: f32) -> gpui::Div {
    let zoom_pct = (zoom * 100.0) as u32;
    let text = format!("{status}  ·  {zoom_pct}%");

    div()
        .h(px(28.0))
        .w_full()
        .bg(rgb(0xFAFAFAu32))
        .border_t_1()
        .border_color(rgb(0xE0E0E0u32))
        .flex()
        .items_center()
        .px(px(16.0))
        .child(div().text_size(px(12.0)).text_color(rgb(0x666666u32)).child(text))
}

// ── Node element ────────────────────────────────────────────────────

fn render_node(
    node: &Node,
    zoom: f32,
    selected: Option<&str>,
    cx: &mut Context<GraphViewPanel>,
) -> gpui::Div {
    let is_selected = selected == Some(node.id.as_str());
    let text_rgb = parse_hex_u32(&node.style.text_color);
    let subtext_rgb = parse_hex_u32(&node.style.subtext_color);

    let (fill, border_color) = if is_selected {
        (parse_color("#E8F5E9"), parse_color("#43A047"))
    } else {
        (parse_color(&node.style.fill), parse_color(&node.style.border))
    };

    let border_w = if is_selected { 2.5 } else { node.style.border_width };

    let radius = match node.shape {
        NodeShape::RoundedRect => px(node.style.border_radius * zoom),
        NodeShape::Rect => px(0.0),
        NodeShape::Circle | NodeShape::Ellipse | NodeShape::Pill => px(node.height * zoom / 2.0),
        NodeShape::Diamond => px(4.0 * zoom),
    };

    let label = match &node.icon {
        Some(icon) => format!("{icon} {}", node.label),
        None => node.label.clone(),
    };

    let mut el = div()
        .absolute()
        .left(px(node.x * zoom))
        .top(px(node.y * zoom))
        .w(px(node.width * zoom))
        .h(px(node.height * zoom))
        .bg(fill)
        .border(px(border_w * zoom))
        .border_color(border_color)
        .rounded(radius)
        .cursor_pointer()
        .flex()
        .flex_col()
        .justify_center()
        .px(px(10.0 * zoom))
        .overflow_hidden()
        .child(
            div()
                .text_size(px(node.style.font_size * zoom))
                .text_color(rgb(text_rgb))
                .child(label),
        );

    if let Some(sublabel) = &node.sublabel {
        el = el.child(
            div()
                .text_size(px(node.style.subfont_size * zoom))
                .text_color(rgb(subtext_rgb))
                .child(sublabel.clone()),
        );
    }

    // Hover effect
    el = el.hover(|style| style.border_color(rgb(0x42A5F5u32)));

    // Left click — select; double-click — drill down
    {
        let node_id = node.id.clone();
        let click_action = node.on_click.clone();
        let dbl_action = node.on_double_click.clone();
        el = el.on_mouse_down(MouseButton::Left, cx.listener(move |this, ev: &gpui::MouseDownEvent, _window, cx| {
            if ev.click_count == 2 {
                // Double click → drill down
                if let Some(Action::DrillDown { target }) = &dbl_action {
                    if let Some(new_view) = get_drill_down_view(target) {
                        this.push_view(new_view);
                        cx.notify();
                        return;
                    }
                }
            }
            // Single click → select
            this.selected = Some(node_id.clone());
            if let Some(action) = &click_action {
                handle_action(action);
            }
            cx.notify();
        }));
    }

    // Right click — highlight connections
    {
        let node_id = node.id.clone();
        el = el.on_mouse_down(MouseButton::Right, cx.listener(move |this, _ev, _window, cx| {
            if this.selected.as_deref() == Some(&node_id) {
                this.selected = None; // toggle off
            } else {
                this.selected = Some(node_id.clone());
            }
            cx.notify();
        }));
    }

    el
}

fn handle_action(action: &Action) {
    match action {
        Action::OpenFile { file, line } => {
            tracing::info!(file = %file, line = line, "open file");
        }
        Action::DrillDown { target } => {
            tracing::info!(target = %target, "drill down");
        }
        Action::Back => {
            tracing::info!("navigate back");
        }
        Action::ShowInfo { target } => {
            tracing::info!(target = %target, "show info");
        }
        Action::Highlight { target } => {
            tracing::info!(target = %target, "highlight");
        }
    }
}

/// Map a drill-down target to a fake view (for testing).
fn get_drill_down_view(target: &str) -> Option<ViewSpec> {
    match target {
        "atrium-agent-sdk" => Some(fake_data::agent_sdk_module_view()),
        "transport" => Some(fake_data::transport_symbol_view()),
        _ => {
            tracing::info!(target = %target, "no drill-down view available");
            None
        }
    }
}

// ── Canvas: edges + groups ──────────────────────────────────────────

fn parse_color(hex_str: &str) -> Hsla {
    let hex = hex_str.trim_start_matches('#');
    let val = u32::from_str_radix(hex, 16).unwrap_or(0xCCCCCC);
    rgb(val).into()
}

fn parse_hex_u32(hex_str: &str) -> u32 {
    let hex = hex_str.trim_start_matches('#');
    u32::from_str_radix(hex, 16).unwrap_or(0x333333)
}

fn quad(
    bounds: Bounds<Pixels>,
    corner_radii: Corners<Pixels>,
    background: Hsla,
    border_width: Pixels,
    border_color: Hsla,
) -> PaintQuad {
    PaintQuad {
        bounds,
        corner_radii,
        background: background.into(),
        border_widths: Edges::all(border_width),
        border_color,
        border_style: BorderStyle::Solid,
    }
}

fn draw_edge(window: &mut Window, edge: &Edge, zoom: f32, highlight: bool) {
    let color = if highlight {
        parse_color("#1E88E5")
    } else {
        parse_color(&edge.style.color)
    };
    let width = if highlight { edge.style.width * 2.5 } else { edge.style.width };
    if edge.points.len() < 2 {
        return;
    }
    for pair in edge.points.windows(2) {
        paint_line(window, pair[0].x * zoom, pair[0].y * zoom, pair[1].x * zoom, pair[1].y * zoom, width * zoom, color);
    }
    if edge.style.arrow {
        let last = &edge.points[edge.points.len() - 1];
        draw_dot(window, last.x * zoom, last.y * zoom, 8.0 * zoom, color);
    }
}

fn draw_group(window: &mut Window, group: &Group, zoom: f32) {
    let fill = parse_color(&group.style.fill);
    let border_color = parse_color(&group.style.border);
    let bounds = Bounds {
        origin: point(px(group.x * zoom), px(group.y * zoom)),
        size: size(px(group.width * zoom), px(group.height * zoom)),
    };
    window.paint_quad(quad(bounds, Corners::all(px(group.style.border_radius * zoom)), fill, px(zoom), border_color));
}

fn paint_line(window: &mut Window, x1: f32, y1: f32, x2: f32, y2: f32, width: f32, color: Hsla) {
    let (min_x, max_x) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
    let (min_y, max_y) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
    let w = max_x - min_x;
    let h = max_y - min_y;
    let (bx, by, bw, bh) = if w > h {
        (min_x, min_y - width / 2.0, w, width)
    } else {
        (min_x - width / 2.0, min_y, width, h)
    };
    let bounds = Bounds {
        origin: point(px(bx), px(by)),
        size: size(px(bw), px(bh)),
    };
    window.paint_quad(quad(bounds, Corners::default(), color, px(0.0), transparent_black()));
}

fn draw_dot(window: &mut Window, x: f32, y: f32, diameter: f32, color: Hsla) {
    let r = diameter / 2.0;
    let bounds = Bounds {
        origin: point(px(x - r), px(y - r)),
        size: size(px(diameter), px(diameter)),
    };
    window.paint_quad(quad(bounds, Corners::all(px(r)), color, px(0.0), transparent_black()));
}
