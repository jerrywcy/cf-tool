use super::View;

pub enum ComponentMsg {
    AppClose,
    ChangedTo(usize),
    EnterNewView(Box<dyn View>),
    ExitCurrentView,
    ChangeToTab(usize),
    Locked,
    Update,
    None,
}

pub enum ViewMsg {
    AppClose,
    EnterNewView(Box<dyn View>),
    ExitCurrentView,
    None,
}
