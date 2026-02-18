use crate::editor::core::{EditorCommand, KeyChord, KeyModifier};

pub(crate) fn char_to_byte_index(content: &str, char_index: usize) -> usize {
    content
        .char_indices()
        .nth(char_index)
        .map(|(byte_index, _)| byte_index)
        .unwrap_or(content.len())
}

pub(crate) fn line_to_char_index(content: &str, line: usize) -> usize {
    if line == 0 {
        return 0;
    }
    let mut char_index = 0;
    let mut current_line = 0;

    for ch in content.chars() {
        char_index += 1;
        if ch == '\n' {
            current_line += 1;
            if current_line == line {
                return char_index;
            }
        }
    }

    char_index
}

pub(crate) fn default_shortcut_bindings() -> Vec<(KeyChord, EditorCommand)> {
    vec![
        (
            KeyChord::new("S", vec![KeyModifier::CmdOrCtrl]),
            EditorCommand::Save,
        ),
        (
            KeyChord::new("Z", vec![KeyModifier::CmdOrCtrl]),
            EditorCommand::Undo,
        ),
        (
            KeyChord::new("Z", vec![KeyModifier::CmdOrCtrl, KeyModifier::Shift]),
            EditorCommand::Redo,
        ),
        (
            KeyChord::new("Y", vec![KeyModifier::CmdOrCtrl]),
            EditorCommand::Redo,
        ),
        (
            KeyChord::new("P", vec![KeyModifier::CmdOrCtrl]),
            EditorCommand::QuickOpen,
        ),
        (
            KeyChord::new("F", vec![KeyModifier::CmdOrCtrl]),
            EditorCommand::FindInFile,
        ),
        (
            KeyChord::new("F", vec![KeyModifier::CmdOrCtrl, KeyModifier::Shift]),
            EditorCommand::FindInWorkspace,
        ),
    ]
}
