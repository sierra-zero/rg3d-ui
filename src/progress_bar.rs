use crate::{
    canvas::CanvasBuilder,
    border::BorderBuilder,
    message::{
        UiMessage,
        UiMessageData,
        WidgetMessage,
        WidgetProperty
    },
    node::UINode,
    widget::{
        Widget,
        WidgetBuilder,
    },
    Control,
    UserInterface,
    core::{
        pool::Handle,
        math::vec2::Vec2,
        color::Color
    },
    brush::Brush,
};

pub struct ProgressBar<M: 'static, C: 'static + Control<M, C>> {
    widget: Widget<M, C>,
    progress: f32,
    indicator: Handle<UINode<M, C>>,
    body: Handle<UINode<M, C>>
}

impl<M, C: 'static + Control<M, C>> Control<M, C> for ProgressBar<M, C> {
    fn widget(&self) -> &Widget<M, C> {
        &self.widget
    }

    fn widget_mut(&mut self) -> &mut Widget<M, C> {
        &mut self.widget
    }

    fn raw_copy(&self) -> UINode<M, C> {
        UINode::ProgressBar(Self {
            widget: self.widget.raw_copy(),
            progress: self.progress,
            indicator: self.indicator,
            body: self.body
        })
    }

    fn arrange_override(&self, ui: &UserInterface<M, C>, final_size: Vec2) -> Vec2 {
        let size = self.widget.arrange_override(ui, final_size);

        self.widget.post_message(
            UiMessage::targeted(self.indicator,
                UiMessageData::Widget(
                    WidgetMessage::Property(
                        WidgetProperty::Width(size.x * self.progress)))));

        self.widget.post_message(
            UiMessage::targeted(self.indicator,
                UiMessageData::Widget(
                    WidgetMessage::Property(
                        WidgetProperty::Height(size.y)))));

        size
    }

    fn handle_message(&mut self, self_handle: Handle<UINode<M, C>>, ui: &mut UserInterface<M, C>, message: &mut UiMessage<M, C>) {
        self.widget.handle_message(self_handle, ui, message);
    }
}

impl<M: 'static, C: 'static + Control<M, C>> ProgressBar<M, C> {
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.min(1.0).max(0.0);
        self.widget.invalidate_layout();
    }

    pub fn progress(&self) -> f32 {
        self.progress
    }
}

pub struct ProgressBarBuilder<M: 'static, C: 'static + Control<M, C>> {
    widget_builder: WidgetBuilder<M, C>,
    body: Option<Handle<UINode<M, C>>>,
    indicator: Option<Handle<UINode<M, C>>>,
    progress: f32,
}

impl<M: 'static, C: 'static + Control<M, C>> ProgressBarBuilder<M, C> {
    pub fn new(widget_builder: WidgetBuilder<M, C>) -> Self {
        Self {
            widget_builder,
            body: None,
            indicator: None,
            progress: 0.0,
        }
    }

    pub fn with_body(mut self, body: Handle<UINode<M, C>>) -> Self {
        self.body = Some(body);
        self
    }

    pub fn with_indicator(mut self, indicator: Handle<UINode<M, C>>) -> Self {
        self.indicator = Some(indicator);
        self
    }

    pub fn with_progress(mut self, progress: f32) -> Self {
        self.progress = progress.min(1.0).max(0.0);
        self
    }

    pub fn build(self, ui: &mut UserInterface<M, C>) -> Handle<UINode<M, C>> {
        let body = self.body.unwrap_or_else(|| {
            BorderBuilder::new(WidgetBuilder::new())
                .build(ui)
        });

        let indicator = self.indicator.unwrap_or_else(|| {
            BorderBuilder::new(WidgetBuilder::new()
                .with_background(Brush::Solid(Color::opaque(180, 180, 180))))
                .build(ui)
        });

        let canvas = CanvasBuilder::new(WidgetBuilder::new()
            .with_child(indicator))
            .build(ui);

        ui.link_nodes(canvas, body);

        let progress_bar = ProgressBar {
            widget: self.widget_builder
                .with_child(body)
                .build(),
            progress: self.progress,
            indicator,
            body,
        };

        ui.add_node(UINode::ProgressBar(progress_bar))
    }
}

