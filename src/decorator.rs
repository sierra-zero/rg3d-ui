use std::{
    rc::Rc,
    any::Any
};
use crate::{
    core::{
        color::Color,
        pool::Handle,
        math::{
            vec2::Vec2,
            Rect
        }
    },
    style::Style,
    border::Border,
    message::{
        UiMessage,
        UiMessageData,
        WidgetMessage,
    },
    node::UINode,
    Control,
    UserInterface,
    widget::Widget,
    draw::DrawingContext,
    brush::{
        Brush,
        GradientPoint
    },
    border::BorderBuilder,
};

pub struct Decorator<M: 'static, C: 'static + Control<M, C>> {
    border: Border<M, C>,
    normal_brush: Brush,
    hover_brush: Brush,
    pressed_brush: Brush,
}

impl<M: 'static, C: 'static + Control<M, C>> Control<M, C> for Decorator<M, C> {
    fn widget(&self) -> &Widget<M, C> {
        self.border.widget()
    }

    fn widget_mut(&mut self) -> &mut Widget<M, C> {
        self.border.widget_mut()
    }

    fn measure_override(&self, ui: &UserInterface<M, C>, available_size: Vec2) -> Vec2 {
        self.border.measure_override(ui, available_size)
    }

    fn arrange_override(&self, ui: &UserInterface<M, C>, final_size: Vec2) -> Vec2 {
        self.border.arrange_override(ui, final_size)
    }

    fn arrange(&self, ui: &UserInterface<M, C>, final_rect: &Rect<f32>) {
        self.border.arrange(ui, final_rect)
    }

    fn is_measure_valid(&self, ui: &UserInterface<M, C>) -> bool {
        self.border.is_measure_valid(ui)
    }

    fn is_arrange_valid(&self, ui: &UserInterface<M, C>) -> bool {
        self.border.is_arrange_valid(ui)
    }

    fn measure(&self, ui: &UserInterface<M, C>, available_size: Vec2) {
       self.border.measure(ui, available_size);
    }

    fn draw(&self, drawing_context: &mut DrawingContext) {
        self.border.draw(drawing_context)
    }

    fn update(&mut self, dt: f32) {
        self.border.update(dt)
    }

    fn set_property(&mut self, name: &str, value: &dyn Any) {
        self.border.set_property(name, value);
    }

    fn get_property(&self, name: &str) -> Option<&'_ dyn Any> {
        self.border.get_property(name)
    }

    fn handle_message(&mut self, self_handle: Handle<UINode<M, C>>, ui: &mut UserInterface<M, C>, message: &mut UiMessage<M, C>) {
        self.border.handle_message(self_handle, ui, message);

        match &message.data {
            UiMessageData::Widget(msg) => {
                if message.source == self_handle || self.widget().has_descendant(message.source, ui) {
                    match msg {
                        WidgetMessage::MouseLeave => {
                            self.border
                                .widget_mut()
                                .set_background(self.normal_brush.clone());
                        },
                        WidgetMessage::MouseEnter => {
                            self.border
                                .widget_mut()
                                .set_background(self.hover_brush.clone());
                        },
                        WidgetMessage::MouseDown { .. } => {
                            self.border
                                .widget_mut()
                                .set_background(self.pressed_brush.clone());
                        },
                        WidgetMessage::MouseUp { .. } => {
                            self.border
                                .widget_mut()
                                .set_background(self.normal_brush.clone());
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn apply_style(&mut self, style: Rc<Style>) {
        self.border.apply_style(style)
    }

    fn remove_ref(&mut self, handle: Handle<UINode<M, C>>) {
        self.border.remove_ref(handle)
    }
}

pub struct DecoratorBuilder<M: 'static, C: 'static + Control<M, C>> {
    border_builder: BorderBuilder<M, C>,
    normal_brush: Option<Brush>,
    hover_brush: Option<Brush>,
    pressed_brush: Option<Brush>,
}

impl<M: 'static, C: 'static + Control<M, C>> DecoratorBuilder<M, C> {
    pub fn new(border_builder: BorderBuilder<M, C>) -> Self {
        Self {
            border_builder,
            normal_brush: None,
            hover_brush: None,
            pressed_brush: None
        }
    }

    pub fn with_normal_brush(mut self, brush: Brush) -> Self {
        self.normal_brush = Some(brush);
        self
    }

    pub fn with_hover_brush(mut self, brush: Brush) -> Self {
        self.hover_brush = Some(brush);
        self
    }

    pub fn with_pressed_brush(mut self, brush: Brush) -> Self {
        self.pressed_brush = Some(brush);
        self
    }

    pub fn build(self, ui: &mut UserInterface<M, C>) -> Handle<UINode<M, C>> {
        let normal_brush =  self.normal_brush.unwrap_or_else(|| {
            Brush::LinearGradient {
                from: Vec2::new(0.5, 0.0),
                to: Vec2::new(0.5, 1.0),
                stops: vec![
                    GradientPoint { stop: 0.0, color: Color::opaque(85, 85, 85) },
                    GradientPoint { stop: 0.46, color: Color::opaque(85, 85, 85) },
                    GradientPoint { stop: 0.5, color: Color::opaque(65, 65, 65) },
                    GradientPoint { stop: 0.54, color: Color::opaque(75, 75, 75) },
                    GradientPoint { stop: 1.0, color: Color::opaque(75, 75, 75) },
                ],
            }
        });

        let mut border = self.border_builder.build_node();

        border.widget_mut().set_background(normal_brush.clone());

        let decorator = UINode::Decorator(Decorator {
            border,
            normal_brush,
            hover_brush: self.hover_brush.unwrap_or_else(|| {
                Brush::LinearGradient {
                    from: Vec2::new(0.5, 0.0),
                    to: Vec2::new(0.5, 1.0),
                    stops: vec![
                        GradientPoint { stop: 0.0, color: Color::opaque(105, 95, 85) },
                        GradientPoint { stop: 0.46, color: Color::opaque(105, 95, 85) },
                        GradientPoint { stop: 0.5, color: Color::opaque(85, 75, 65) },
                        GradientPoint { stop: 0.54, color: Color::opaque(95, 85, 75) },
                        GradientPoint { stop: 1.0, color: Color::opaque(95, 85, 75) },
                    ],
                }
            }),
            pressed_brush: self.pressed_brush.unwrap_or_else(|| {
                Brush::LinearGradient {
                    from: Vec2::new(0.5, 0.0),
                    to: Vec2::new(0.5, 1.0),
                    stops: vec![
                        GradientPoint { stop: 0.0, color: Color::opaque(65, 65, 65) },
                        GradientPoint { stop: 0.46, color: Color::opaque(65, 65, 65) },
                        GradientPoint { stop: 0.5, color: Color::opaque(45, 45, 45) },
                        GradientPoint { stop: 0.54, color: Color::opaque(55, 55, 55) },
                        GradientPoint { stop: 1.0, color: Color::opaque(55, 55, 55) },
                    ],
                }
            })
        });

        let handle = ui.add_node(decorator);

        ui.flush_messages();

        handle
    }
}