use bevy_egui::egui;

/// Compute a slider range centered on the current value.
///
/// For zero or near-zero: [-1, 1].
/// For negative values: [2*val, 0] (or [2*val, -2*val] if very negative).
/// For positive values: [0, 2*val].
pub fn smart_slider_range(value: f32) -> (f32, f32) {
    let abs_val = value.abs();
    if abs_val < 0.001 {
        return (-1.0, 1.0);
    }
    let extent = abs_val * 2.0;
    if value < 0.0 {
        (-extent, extent)
    } else {
        (0.0, extent)
    }
}

/// Helper to update a #define value in the source string.
pub fn update_define_in_source(source: &str, key: &str, new_value: f32) -> String {
    let mut new_lines = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#define") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == key {
                new_lines.push(format!("#define {} {}", key, new_value));
                continue;
            }
        }
        new_lines.push(line.to_string());
    }

    new_lines.join("\n")
}

// --- Syntax Highlighting ---

const HL_COMMENT: egui::Color32 = egui::Color32::from_rgb(0x6A, 0x99, 0x55);
const HL_DIRECTIVE: egui::Color32 = egui::Color32::from_rgb(0xC5, 0x86, 0xC0);
const HL_KEYWORD: egui::Color32 = egui::Color32::from_rgb(0x56, 0x9C, 0xD6);
const HL_RULE_LABEL: egui::Color32 = egui::Color32::from_rgb(0x4E, 0xC9, 0xB0);
const HL_NUMBER: egui::Color32 = egui::Color32::from_rgb(0xB5, 0xCE, 0xA8);
const HL_ARROW: egui::Color32 = egui::Color32::from_rgb(0xD4, 0xD4, 0xD4);
const HL_BRACKET: egui::Color32 = egui::Color32::from_rgb(0xDA, 0xDA, 0x6E);
const HL_SYMBOL: egui::Color32 = egui::Color32::from_rgb(0x9C, 0xDC, 0xFE);
const HL_SPECIAL: egui::Color32 = egui::Color32::from_rgb(0xCE, 0x91, 0x78);
const HL_DEFAULT: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);

pub fn highlight_lsystem(text: &str, font_id: egui::FontId) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob {
        text: text.to_string(),
        ..Default::default()
    };

    let mut pos = 0;
    for line in text.split_inclusive('\n') {
        let line_end = pos + line.len();
        let trimmed = line.trim();
        let ws = line.len() - line.trim_start().len();
        let content_start = pos + ws;

        if trimmed.is_empty() {
            push_hl(&mut job, pos, line_end, HL_DEFAULT, &font_id);
        } else if trimmed.starts_with("//") {
            push_hl(&mut job, pos, line_end, HL_COMMENT, &font_id);
        } else if trimmed.starts_with('#') {
            if ws > 0 {
                push_hl(&mut job, pos, content_start, HL_DEFAULT, &font_id);
            }
            let kw_end = trimmed
                .find(|c: char| c == ':' || c.is_ascii_whitespace())
                .unwrap_or(trimmed.len());
            push_hl(
                &mut job,
                content_start,
                content_start + kw_end,
                HL_DIRECTIVE,
                &font_id,
            );
            highlight_body(&mut job, text, content_start + kw_end, line_end, &font_id);
        } else if trimmed.starts_with("omega:") {
            if ws > 0 {
                push_hl(&mut job, pos, content_start, HL_DEFAULT, &font_id);
            }
            let kw_len = "omega:".len();
            push_hl(
                &mut job,
                content_start,
                content_start + kw_len,
                HL_KEYWORD,
                &font_id,
            );
            highlight_body(&mut job, text, content_start + kw_len, line_end, &font_id);
        } else if let Some(colon) = trimmed.find(':') {
            // Check for rule label pattern: pN:
            let prefix = &trimmed[..colon];
            if prefix.starts_with('p')
                && prefix.len() > 1
                && prefix[1..].chars().all(|c| c.is_ascii_digit())
            {
                if ws > 0 {
                    push_hl(&mut job, pos, content_start, HL_DEFAULT, &font_id);
                }
                let label_len = colon + 1;
                push_hl(
                    &mut job,
                    content_start,
                    content_start + label_len,
                    HL_RULE_LABEL,
                    &font_id,
                );
                highlight_body(
                    &mut job,
                    text,
                    content_start + label_len,
                    line_end,
                    &font_id,
                );
            } else {
                if ws > 0 {
                    push_hl(&mut job, pos, content_start, HL_DEFAULT, &font_id);
                }
                highlight_body(&mut job, text, content_start, line_end, &font_id);
            }
        } else {
            if ws > 0 {
                push_hl(&mut job, pos, content_start, HL_DEFAULT, &font_id);
            }
            highlight_body(&mut job, text, content_start, line_end, &font_id);
        }

        pos = line_end;
    }

    // Handle text not ending with newline (split_inclusive still yields it, but
    // ensure we haven't missed trailing content).
    if pos < text.len() {
        push_hl(&mut job, pos, text.len(), HL_DEFAULT, &font_id);
    }

    job
}

/// Token-level highlighting for rule/axiom body content.
pub fn highlight_body(
    job: &mut egui::text::LayoutJob,
    text: &str,
    start: usize,
    end: usize,
    font_id: &egui::FontId,
) {
    if start >= end {
        return;
    }

    let bytes = text.as_bytes();
    let mut i = start;

    while i < end {
        let b = bytes[i];

        // Arrow ->
        if b == b'-' && i + 1 < end && bytes[i + 1] == b'>' {
            push_hl(job, i, i + 2, HL_ARROW, font_id);
            i += 2;
            continue;
        }

        // Numbers
        if b.is_ascii_digit() {
            let s = i;
            while i < end
                && (bytes[i].is_ascii_digit()
                    || (bytes[i] == b'.' && i + 1 < end && bytes[i + 1].is_ascii_digit()))
            {
                i += 1;
            }
            push_hl(job, s, i, HL_NUMBER, font_id);
            continue;
        }

        // Brackets
        if b == b'[' || b == b']' {
            push_hl(job, i, i + 1, HL_BRACKET, font_id);
            i += 1;
            continue;
        }

        // Turtle symbols
        if b"Ff+-&^/\\|$".contains(&b) {
            push_hl(job, i, i + 1, HL_SYMBOL, font_id);
            i += 1;
            continue;
        }

        // Prop / material / color / width symbols
        if b"~,';!".contains(&b) {
            push_hl(job, i, i + 1, HL_SPECIAL, font_id);
            i += 1;
            continue;
        }

        // Default run: accumulate until next token
        let s = i;
        while i < end {
            let c = bytes[i];
            if c == b'-' && i + 1 < end && bytes[i + 1] == b'>' {
                break;
            }
            if c.is_ascii_digit() || b"[]Ff+-&^/\\|$~,';!".contains(&c) {
                break;
            }
            i += 1;
        }
        if s < i {
            push_hl(job, s, i, HL_DEFAULT, font_id);
        }
    }
}

pub fn push_hl(
    job: &mut egui::text::LayoutJob,
    start: usize,
    end: usize,
    color: egui::Color32,
    font_id: &egui::FontId,
) {
    if start >= end {
        return;
    }
    job.sections.push(egui::text::LayoutSection {
        leading_space: 0.0,
        byte_range: start..end,
        format: egui::TextFormat::simple(font_id.clone(), color),
    });
}
