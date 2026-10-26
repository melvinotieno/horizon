pub mod button;

use std::time::Duration;

use horizon_theme::{Colors, Tokens};
use vello::kurbo::Rect;

/// A unique identifier for a widget during its lifetime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(pub(crate) taffy::NodeId);

/// The kind of action that can be triggered by a widget.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionKind {
    Clicked,
}

/// An interaction triggered by a widget.
#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    pub source: WidgetId,
    pub name: Option<&'static str>,
    pub kind: ActionKind,
}

/// The state flags for a widget, indicating its current interaction state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Flags {
    pub hovered: bool,
    pub within: bool,
    pub active: bool,
    pub focused: bool,
    pub disabled: bool,
}

/// The requests a widget can make to the system.
#[derive(Debug, Default)]
pub(crate) struct Requests {
    pub paint: bool,
    pub layout: Vec<taffy::NodeId>,
    pub anim_frame: bool,
    pub actions: Vec<Action>,
    pub focus: Option<taffy::NodeId>,
    pub resign_focus: bool,
    pub clipboard_write: Option<String>,
    pub clipboard_read: bool,
    pub reposition: bool,
    pub reveal: Option<(taffy::NodeId, Rect)>,
    pub wake_at: Option<Duration>,
}

/// The environment provided to a widget during its update and paint phases.
#[derive(Clone, Copy)]
pub struct WidgetEnv<'a> {
    pub tokens: &'a Tokens,
    pub colors: &'a Colors,
    pub elapsed: Duration,
    pub scale: f64,
    pub reduced_motion: bool,
}
