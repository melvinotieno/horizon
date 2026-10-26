/// The pointer image a widget wants to display when hovered.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Cursor {
    /// The default pointer image, typically an arrow.
    #[default]
    Default,
    /// A hand indicating a clickable item, typically a link.
    Pointer,
    /// A text selection pointer, typically an I-beam.
    Text,
    /// An open hand for something that can be dragged.
    Grab,
    /// A closed hand for something that is being dragged.
    Grabbing,
}

/// A mouse or touchpad button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButton {
    /// Usually the the left button.
    Primary,
    /// Usually the right button.
    Secondary,
    /// The middle button, often a scroll wheel.
    Middle,
}

/// The modifier keys currently pressed during an event.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub caps: bool,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}
