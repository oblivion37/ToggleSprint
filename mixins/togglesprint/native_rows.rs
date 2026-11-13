fn sprint_settings_rows(
    options: &OptionsUi,
    settings: &Settings,
    index: usize,
) -> core::ops::ControlFlow<Vec<Row>> {
    let Some(entry) = settings.mods.installed.get(index).filter(|m| m.id == "togglesprint") else {
        return core::ops::ControlFlow::Continue(());
    };
    let Some(page) = &entry.native else {
        return core::ops::ControlFlow::Continue(());
    };

    let mut rows = vec![Row::Note(entry.title.clone())];
    for (row, label) in page.buttons.iter().enumerate() {
        rows.push(Row::Buttons(
            button(0.0, 0.0, 310.0, Text::literal(label.clone()), Act::NativeButton(index, row)),
            None,
        ));
    }
    // The native settings API provides the toggle state through its button label.
    let crouch_enabled = page.buttons.get(1).is_some_and(|label| label == "ToggleCrouch: ON");
    for (row, [name, label, default]) in page.keys.iter().enumerate() {
        let binding = settings
            .mods
            .keys
            .get(&entry.id)
            .and_then(|keys| keys.get(name))
            .unwrap_or(default);
        let binding = keybinds::display_name(binding);
        let active = name != "toggle_crouch" || crouch_enabled;
        let message = if active && options.selected_native_key == Some((index, row)) {
            Text::literal(format!("{label}: > {} <", binding.plain())).with_color(text::YELLOW)
        } else {
            Text::literal(format!("{label}: {}", binding.plain()))
        };
        let widget = if active {
            button(0.0, 0.0, 310.0, message, Act::NativeKey(index, row))
        } else {
            inactive(button(0.0, 0.0, 310.0, message, Act::None))
        };
        rows.push(Row::Buttons(widget, None));
    }
    core::ops::ControlFlow::Break(rows)
}
