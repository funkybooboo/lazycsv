//! Right-click context menu types.

/// Right-click context menu state
#[derive(Debug, Clone)]
pub struct ContextMenu {
    /// Terminal position where the menu should appear
    pub x: u16,
    pub y: u16,
    /// Currently highlighted menu item index
    pub selected: usize,
    /// Menu items
    pub items: Vec<ContextMenuItem>,
}

/// A single context menu item
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContextMenuItem {
    Cut,
    Copy,
    Paste,
    Separator,
    Clear,
    ColumnDelete,
    ColumnInsertBefore,
    ColumnInsertAfter,
    RowDelete,
    RowInsertAbove,
    RowInsertBelow,
}

impl ContextMenuItem {
    pub fn label(self) -> &'static str {
        match self {
            ContextMenuItem::Cut => "Cut",
            ContextMenuItem::Copy => "Copy",
            ContextMenuItem::Paste => "Paste",
            ContextMenuItem::Clear => "Clear",
            ContextMenuItem::Separator => "",
            ContextMenuItem::ColumnDelete => "Delete",
            ContextMenuItem::ColumnInsertBefore => "Insert Before",
            ContextMenuItem::ColumnInsertAfter => "Insert After",
            ContextMenuItem::RowDelete => "Delete",
            ContextMenuItem::RowInsertAbove => "Insert Above",
            ContextMenuItem::RowInsertBelow => "Insert Below",
        }
    }
}
