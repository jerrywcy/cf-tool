use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

pub fn is_key(key: &KeyEvent, code: KeyCode, modifiers: KeyModifiers) -> bool {
    key.code == code && key.modifiers == modifiers
}

pub fn is_up_key(evt: &KeyEvent) -> bool {
    is_key(evt, KeyCode::Char('k'), KeyModifiers::NONE)
        || is_key(evt, KeyCode::Up, KeyModifiers::NONE)
}

pub fn is_down_key(evt: &KeyEvent) -> bool {
    is_key(evt, KeyCode::Char('j'), KeyModifiers::NONE)
        || is_key(evt, KeyCode::Down, KeyModifiers::NONE)
}

pub fn is_right_key(evt: &KeyEvent) -> bool {
    is_key(evt, KeyCode::Char('l'), KeyModifiers::NONE)
        || is_key(evt, KeyCode::Right, KeyModifiers::NONE)
        || is_key(evt, KeyCode::Tab, KeyModifiers::NONE)
}

pub fn is_left_key(evt: &KeyEvent) -> bool {
    is_key(evt, KeyCode::Char('h'), KeyModifiers::NONE)
        || is_key(evt, KeyCode::Left, KeyModifiers::NONE)
        || is_key(evt, KeyCode::Tab, KeyModifiers::SHIFT)
}

pub fn is_refresh_key(evt: &KeyEvent) -> bool {
    is_key(evt, KeyCode::F(5), KeyModifiers::NONE)
}

pub fn is_enter_key(evt: &KeyEvent) -> bool {
    is_key(evt, KeyCode::Enter, KeyModifiers::NONE)
}

pub fn is_exit_key(evt: &KeyEvent) -> bool {
    is_key(evt, KeyCode::Char('q'), KeyModifiers::NONE)
        || is_key(evt, KeyCode::Esc, KeyModifiers::NONE)
}

pub fn is_terminate_key(evt: &KeyEvent) -> bool {
    is_key(evt, KeyCode::Char('c'), KeyModifiers::CONTROL)
}

pub fn is_scroll_up(evt: &MouseEvent) -> bool {
    evt.kind == MouseEventKind::ScrollUp
}

pub fn is_scroll_down(evt: &MouseEvent) -> bool {
    evt.kind == MouseEventKind::ScrollDown
}
