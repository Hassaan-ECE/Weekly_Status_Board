use crate::model::{BoardDocument, Column as ModelColumn};
use crate::theme::Theme;
use crate::ui::app::{Editing, Selection, StatusApp};
use crate::ui::column::Column;
use crate::ui::input::TextInput;
use chrono::NaiveDate;
use gpui::{
    canvas, div, prelude::*, px, AnyElement, App, Bounds, DispatchPhase, Element, ElementId,
    Entity, GlobalElementId, IntoElement, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Pixels, Window,
};

struct ZoomRem {
    zoom: f32,
    child: Option<AnyElement>,
}

impl IntoElement for ZoomRem {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for ZoomRem {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let rem = px(16.0 * self.zoom);
        let mut child = self.child.take().expect("ZoomRem child");
        let layout_id = window.with_rem_size(Some(rem), |window| child.request_layout(window, cx));
        (layout_id, child)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        _bounds: Bounds<Pixels>,
        child: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let rem = px(16.0 * self.zoom);
        window.with_rem_size(Some(rem), |window| {
            child.prepaint(window, cx);
        });
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        _bounds: Bounds<Pixels>,
        child: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let rem = px(16.0 * self.zoom);
        window.with_rem_size(Some(rem), |window| {
            child.paint(window, cx);
        });
    }
}

fn column_gutter(
    index: usize,
    theme: &Theme,
    view_mode: bool,
    app: Entity<StatusApp>,
) -> impl IntoElement {
    let gutter_id: gpui::SharedString = format!("column-gutter-{index}").into();
    if view_mode {
        return div()
            .id(gutter_id)
            .flex_none()
            .w(px(6.))
            .h_full()
            .into_any_element();
    }

    let app_down = app;
    div()
        .id(gutter_id)
        .flex_none()
        .w(px(6.))
        .h_full()
        .cursor_col_resize()
        .hover(|s| s.bg(theme.border))
        .on_mouse_down(MouseButton::Left, move |event: &MouseDownEvent, _, cx| {
            app_down.update(cx, |app, cx| {
                app.begin_column_resize(index, event.position, cx);
            });
        })
        .into_any_element()
}

#[allow(non_snake_case)]
pub fn Board(
    board: &BoardDocument,
    theme: &Theme,
    today: NaiveDate,
    view_mode: bool,
    editing: &Editing,
    selection: &Selection,
    input: Entity<TextInput>,
    app: Entity<StatusApp>,
) -> impl IntoElement {
    let widths = board.column_widths;
    let zoom = board.zoom;
    let app_bounds = app.clone();
    let app_mouse = app.clone();

    let row = div()
        .id("board-columns")
        .flex()
        .flex_row()
        .size_full()
        .min_h_0()
        .child(Column(
            board,
            theme,
            today,
            ModelColumn::Target,
            widths[0],
            view_mode,
            editing,
            selection,
            input.clone(),
            app.clone(),
        ))
        .child(column_gutter(0, theme, view_mode, app.clone()))
        .child(Column(
            board,
            theme,
            today,
            ModelColumn::InProgress,
            widths[1],
            view_mode,
            editing,
            selection,
            input.clone(),
            app.clone(),
        ))
        .child(column_gutter(1, theme, view_mode, app.clone()))
        .child(Column(
            board,
            theme,
            today,
            ModelColumn::Done,
            widths[2],
            view_mode,
            editing,
            selection,
            input,
            app,
        ));

    div()
        .id("board-workspace")
        .relative()
        .flex_1()
        .w_full()
        .min_h_0()
        .px_3()
        .py_2()
        .bg(theme.background)
        .child(ZoomRem {
            zoom,
            child: Some(row.into_any_element()),
        })
        .child(
            canvas(
                move |bounds, _, cx| {
                    app_bounds.update(cx, |app, _| {
                        app.set_board_bounds(bounds);
                    });
                },
                move |_, _, window, _cx| {
                    let app = app_mouse.clone();
                    window.on_mouse_event({
                        let app = app.clone();
                        move |ev: &MouseMoveEvent, phase, _, cx| {
                            if phase != DispatchPhase::Bubble || !ev.dragging() {
                                return;
                            }
                            app.update(cx, |app, cx| {
                                app.on_column_resize_move(ev.position, cx);
                            });
                        }
                    });
                    window.on_mouse_event(move |_: &MouseUpEvent, phase, _, cx| {
                        if phase != DispatchPhase::Bubble {
                            return;
                        }
                        app.update(cx, |app, cx| {
                            app.end_column_resize(cx);
                        });
                    });
                },
            )
            .absolute()
            .top_0()
            .left_0()
            .size_full(),
        )
}
