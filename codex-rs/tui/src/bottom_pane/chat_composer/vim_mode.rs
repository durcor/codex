use crate::bottom_pane::textarea::TextArea;
use codex_config::types::TuiEditorMode;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use ratatui::style::Stylize;
use ratatui::text::Line;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VimEditingMode {
    Insert,
    Normal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VimOperator {
    Delete,
    Change,
    Yank,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VimFindMotion {
    pub(crate) ch: char,
    pub(crate) forward: bool,
    pub(crate) till: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VimMotion {
    Left,
    Right,
    Up,
    Down,
    NextWordStart,
    NextWordEnd,
    PrevWord,
    LineStart,
    LineEnd,
    FirstNonBlank,
    InputStart,
    InputEnd,
    Find(VimFindMotion),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Pending {
    G,
    Operator(VimOperator),
    Find { forward: bool, till: bool },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VimKeyAction {
    NoOp,
    SwitchToNormal,
    SwitchToInsert,
    InsertAtLineStart,
    Append,
    AppendAtLineEnd,
    OpenBelow,
    OpenAbove,
    Submit,
    Queue,
    Move(VimMotion),
    DeleteChar,
    DeleteMotion(VimMotion),
    DeleteLine,
    DeleteToEndOfLine,
    ChangeMotion(VimMotion),
    ChangeLine,
    ChangeToEndOfLine,
    YankMotion(VimMotion),
    YankLine,
    PasteAfter,
    PasteBefore,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VimState {
    enabled: bool,
    mode: VimEditingMode,
    pending: Option<Pending>,
    last_find: Option<VimFindMotion>,
}

impl VimState {
    pub(crate) fn new(editor_mode: TuiEditorMode) -> Self {
        let enabled = matches!(editor_mode, TuiEditorMode::Vim);
        Self {
            enabled,
            mode: VimEditingMode::Insert,
            pending: None,
            last_find: None,
        }
    }

    pub(crate) fn mode_line(self) -> Option<Line<'static>> {
        if !self.enabled {
            return None;
        }

        Some(match self.mode {
            VimEditingMode::Insert => Line::from("INSERT".green().bold()),
            VimEditingMode::Normal => Line::from("NORMAL".cyan().bold()),
        })
    }

    pub(crate) fn handle_key(&mut self, key_event: KeyEvent) -> Option<VimKeyAction> {
        if !self.enabled || !matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            return None;
        }

        match self.mode {
            VimEditingMode::Insert => self.handle_insert_key(key_event),
            VimEditingMode::Normal => Some(self.handle_normal_key(key_event)),
        }
    }

    fn handle_insert_key(&mut self, key_event: KeyEvent) -> Option<VimKeyAction> {
        if key_event.modifiers == KeyModifiers::NONE && key_event.code == KeyCode::Esc {
            self.mode = VimEditingMode::Normal;
            self.pending = None;
            return Some(VimKeyAction::SwitchToNormal);
        }

        None
    }

    fn handle_normal_key(&mut self, key_event: KeyEvent) -> VimKeyAction {
        if key_event.modifiers == KeyModifiers::NONE && key_event.code == KeyCode::Esc {
            self.pending = None;
            return VimKeyAction::NoOp;
        }

        if let Some(pending) = self.pending.take() {
            return self.handle_pending_key(pending, key_event);
        }

        match key_event {
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                ..
            } => VimKeyAction::Submit,
            KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::NONE,
                ..
            } => VimKeyAction::Queue,
            KeyEvent {
                code: KeyCode::Char('i'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.mode = VimEditingMode::Insert;
                VimKeyAction::SwitchToInsert
            }
            KeyEvent {
                code: KeyCode::Char('I'),
                modifiers: KeyModifiers::SHIFT,
                ..
            } => {
                self.mode = VimEditingMode::Insert;
                VimKeyAction::InsertAtLineStart
            }
            KeyEvent {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.mode = VimEditingMode::Insert;
                VimKeyAction::Append
            }
            KeyEvent {
                code: KeyCode::Char('A'),
                modifiers: KeyModifiers::SHIFT,
                ..
            } => {
                self.mode = VimEditingMode::Insert;
                VimKeyAction::AppendAtLineEnd
            }
            KeyEvent {
                code: KeyCode::Char('o'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.mode = VimEditingMode::Insert;
                VimKeyAction::OpenBelow
            }
            KeyEvent {
                code: KeyCode::Char('O'),
                modifiers: KeyModifiers::SHIFT,
                ..
            } => {
                self.mode = VimEditingMode::Insert;
                VimKeyAction::OpenAbove
            }
            KeyEvent {
                code: KeyCode::Char('h'),
                modifiers: KeyModifiers::NONE,
                ..
            }
            | KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
                ..
            } => VimKeyAction::Move(VimMotion::Left),
            KeyEvent {
                code: KeyCode::Char('j'),
                modifiers: KeyModifiers::NONE,
                ..
            }
            | KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                ..
            } => VimKeyAction::Move(VimMotion::Down),
            KeyEvent {
                code: KeyCode::Char('k'),
                modifiers: KeyModifiers::NONE,
                ..
            }
            | KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                ..
            } => VimKeyAction::Move(VimMotion::Up),
            KeyEvent {
                code: KeyCode::Char('l'),
                modifiers: KeyModifiers::NONE,
                ..
            }
            | KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                ..
            } => VimKeyAction::Move(VimMotion::Right),
            KeyEvent {
                code: KeyCode::Char('w'),
                modifiers: KeyModifiers::NONE,
                ..
            } => VimKeyAction::Move(VimMotion::NextWordStart),
            KeyEvent {
                code: KeyCode::Char('e'),
                modifiers: KeyModifiers::NONE,
                ..
            } => VimKeyAction::Move(VimMotion::NextWordEnd),
            KeyEvent {
                code: KeyCode::Char('b'),
                modifiers: KeyModifiers::NONE,
                ..
            } => VimKeyAction::Move(VimMotion::PrevWord),
            KeyEvent {
                code: KeyCode::Char('0'),
                modifiers: KeyModifiers::NONE,
                ..
            }
            | KeyEvent {
                code: KeyCode::Home,
                modifiers: KeyModifiers::NONE,
                ..
            } => VimKeyAction::Move(VimMotion::LineStart),
            KeyEvent {
                code: KeyCode::Char('$'),
                modifiers: KeyModifiers::SHIFT,
                ..
            }
            | KeyEvent {
                code: KeyCode::End,
                modifiers: KeyModifiers::NONE,
                ..
            } => VimKeyAction::Move(VimMotion::LineEnd),
            KeyEvent {
                code: KeyCode::Char('^'),
                modifiers: KeyModifiers::SHIFT,
                ..
            } => VimKeyAction::Move(VimMotion::FirstNonBlank),
            KeyEvent {
                code: KeyCode::Char('G'),
                modifiers: KeyModifiers::SHIFT,
                ..
            } => VimKeyAction::Move(VimMotion::InputEnd),
            KeyEvent {
                code: KeyCode::Char('g'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.pending = Some(Pending::G);
                VimKeyAction::NoOp
            }
            KeyEvent {
                code: KeyCode::Char('x'),
                modifiers: KeyModifiers::NONE,
                ..
            } => VimKeyAction::DeleteChar,
            KeyEvent {
                code: KeyCode::Char('D'),
                modifiers: KeyModifiers::SHIFT,
                ..
            } => VimKeyAction::DeleteToEndOfLine,
            KeyEvent {
                code: KeyCode::Char('C'),
                modifiers: KeyModifiers::SHIFT,
                ..
            } => {
                self.mode = VimEditingMode::Insert;
                VimKeyAction::ChangeToEndOfLine
            }
            KeyEvent {
                code: KeyCode::Char('Y'),
                modifiers: KeyModifiers::SHIFT,
                ..
            } => VimKeyAction::YankLine,
            KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.pending = Some(Pending::Operator(VimOperator::Delete));
                VimKeyAction::NoOp
            }
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.pending = Some(Pending::Operator(VimOperator::Change));
                VimKeyAction::NoOp
            }
            KeyEvent {
                code: KeyCode::Char('y'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.pending = Some(Pending::Operator(VimOperator::Yank));
                VimKeyAction::NoOp
            }
            KeyEvent {
                code: KeyCode::Char('p'),
                modifiers: KeyModifiers::NONE,
                ..
            } => VimKeyAction::PasteAfter,
            KeyEvent {
                code: KeyCode::Char('P'),
                modifiers: KeyModifiers::SHIFT,
                ..
            } => VimKeyAction::PasteBefore,
            KeyEvent {
                code: KeyCode::Char('f'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.pending = Some(Pending::Find {
                    forward: true,
                    till: false,
                });
                VimKeyAction::NoOp
            }
            KeyEvent {
                code: KeyCode::Char('F'),
                modifiers: KeyModifiers::SHIFT,
                ..
            } => {
                self.pending = Some(Pending::Find {
                    forward: false,
                    till: false,
                });
                VimKeyAction::NoOp
            }
            KeyEvent {
                code: KeyCode::Char('t'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.pending = Some(Pending::Find {
                    forward: true,
                    till: true,
                });
                VimKeyAction::NoOp
            }
            KeyEvent {
                code: KeyCode::Char('T'),
                modifiers: KeyModifiers::SHIFT,
                ..
            } => {
                self.pending = Some(Pending::Find {
                    forward: false,
                    till: true,
                });
                VimKeyAction::NoOp
            }
            KeyEvent {
                code: KeyCode::Char(';'),
                modifiers: KeyModifiers::NONE,
                ..
            } => self.last_find.map_or(VimKeyAction::NoOp, |find| {
                VimKeyAction::Move(VimMotion::Find(find))
            }),
            KeyEvent {
                code: KeyCode::Char(','),
                modifiers: KeyModifiers::NONE,
                ..
            } => self.last_find.map_or(VimKeyAction::NoOp, |find| {
                VimKeyAction::Move(VimMotion::Find(VimFindMotion {
                    forward: !find.forward,
                    ..find
                }))
            }),
            _ => VimKeyAction::NoOp,
        }
    }

    fn handle_pending_key(&mut self, pending: Pending, key_event: KeyEvent) -> VimKeyAction {
        match pending {
            Pending::G => match key_event {
                KeyEvent {
                    code: KeyCode::Char('g'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => VimKeyAction::Move(VimMotion::InputStart),
                _ => VimKeyAction::NoOp,
            },
            Pending::Operator(operator) => self.handle_operator_key(operator, key_event),
            Pending::Find { forward, till } => match key_event {
                KeyEvent {
                    code: KeyCode::Char(ch),
                    modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                    ..
                } => {
                    let motion = VimFindMotion { ch, forward, till };
                    self.last_find = Some(motion);
                    VimKeyAction::Move(VimMotion::Find(motion))
                }
                _ => VimKeyAction::NoOp,
            },
        }
    }

    fn handle_operator_key(&mut self, operator: VimOperator, key_event: KeyEvent) -> VimKeyAction {
        match key_event {
            KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::NONE,
                ..
            } if matches!(operator, VimOperator::Delete) => VimKeyAction::DeleteLine,
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::NONE,
                ..
            } if matches!(operator, VimOperator::Change) => {
                self.mode = VimEditingMode::Insert;
                VimKeyAction::ChangeLine
            }
            KeyEvent {
                code: KeyCode::Char('y'),
                modifiers: KeyModifiers::NONE,
                ..
            } if matches!(operator, VimOperator::Yank) => VimKeyAction::YankLine,
            _ => {
                let Some(motion) = Self::motion_for_key(key_event) else {
                    return VimKeyAction::NoOp;
                };

                match operator {
                    VimOperator::Delete => VimKeyAction::DeleteMotion(motion),
                    VimOperator::Change => {
                        self.mode = VimEditingMode::Insert;
                        VimKeyAction::ChangeMotion(motion)
                    }
                    VimOperator::Yank => VimKeyAction::YankMotion(motion),
                }
            }
        }
    }

    fn motion_for_key(key_event: KeyEvent) -> Option<VimMotion> {
        match key_event {
            KeyEvent {
                code: KeyCode::Char('h'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(VimMotion::Left),
            KeyEvent {
                code: KeyCode::Char('j'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(VimMotion::Down),
            KeyEvent {
                code: KeyCode::Char('k'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(VimMotion::Up),
            KeyEvent {
                code: KeyCode::Char('l'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(VimMotion::Right),
            KeyEvent {
                code: KeyCode::Char('w'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(VimMotion::NextWordStart),
            KeyEvent {
                code: KeyCode::Char('e'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(VimMotion::NextWordEnd),
            KeyEvent {
                code: KeyCode::Char('b'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(VimMotion::PrevWord),
            KeyEvent {
                code: KeyCode::Char('0'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(VimMotion::LineStart),
            KeyEvent {
                code: KeyCode::Char('$'),
                modifiers: KeyModifiers::SHIFT,
                ..
            } => Some(VimMotion::LineEnd),
            KeyEvent {
                code: KeyCode::Char('^'),
                modifiers: KeyModifiers::SHIFT,
                ..
            } => Some(VimMotion::FirstNonBlank),
            KeyEvent {
                code: KeyCode::Char('g'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(VimMotion::InputStart),
            KeyEvent {
                code: KeyCode::Char('G'),
                modifiers: KeyModifiers::SHIFT,
                ..
            } => Some(VimMotion::InputEnd),
            _ => None,
        }
    }
}

pub(crate) fn apply_motion(textarea: &TextArea, motion: VimMotion) -> usize {
    match motion {
        VimMotion::Left => {
            let mut clone = clone_textarea(textarea);
            clone.move_cursor_left();
            clone.cursor()
        }
        VimMotion::Right => {
            let mut clone = clone_textarea(textarea);
            clone.move_cursor_right();
            clone.cursor()
        }
        VimMotion::Up => {
            let mut clone = clone_textarea(textarea);
            clone.move_cursor_up();
            clone.cursor()
        }
        VimMotion::Down => {
            let mut clone = clone_textarea(textarea);
            clone.move_cursor_down();
            clone.cursor()
        }
        VimMotion::NextWordStart => textarea.start_of_next_word(),
        VimMotion::NextWordEnd => textarea.end_of_next_word(),
        VimMotion::PrevWord => textarea.beginning_of_previous_word(),
        VimMotion::LineStart => textarea.current_line_start(),
        VimMotion::LineEnd => textarea.current_line_end(),
        VimMotion::FirstNonBlank => textarea.first_non_blank_of_current_line(),
        VimMotion::InputStart => 0,
        VimMotion::InputEnd => textarea.text().len(),
        VimMotion::Find(find) => find_in_line(textarea, find).unwrap_or(textarea.cursor()),
    }
}

fn clone_textarea(textarea: &TextArea) -> TextArea {
    let mut clone = TextArea::new();
    clone.set_text_with_elements(textarea.text(), &textarea.text_elements());
    clone.set_cursor(textarea.cursor());
    clone
}

fn find_in_line(textarea: &TextArea, find: VimFindMotion) -> Option<usize> {
    let line = textarea.current_line_range();
    let text = textarea.text();
    let cursor = textarea.cursor();
    if find.forward {
        let start = cursor.min(line.end);
        let slice = &text[start..line.end];
        let (idx, _) = slice.char_indices().find(|(_, ch)| *ch == find.ch)?;
        let pos = start + idx;
        Some(if find.till {
            text[..pos]
                .char_indices()
                .last()
                .map_or(line.start, |(prev, _)| prev.max(cursor))
        } else {
            pos
        })
    } else {
        let slice = &text[line.start..cursor.min(line.end)];
        let (idx, _) = slice.char_indices().rev().find(|(_, ch)| *ch == find.ch)?;
        let pos = line.start + idx;
        Some(if find.till {
            let ch_len = text[pos..].chars().next()?.len_utf8();
            (pos + ch_len).min(line.end)
        } else {
            pos
        })
    }
}
