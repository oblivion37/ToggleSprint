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
    let [hud_label, crouch_label] = page.buttons.as_slice() else {
        return core::ops::ControlFlow::Continue(());
    };
    let [[name, _, default]] = page.keys.as_slice() else {
        return core::ops::ControlFlow::Continue(());
    };

    let enabled = crouch_label == "ToggleCrouch: ON";
    let binding = settings
        .mods
        .keys
        .get(&entry.id)
        .and_then(|keys| keys.get(name))
        .unwrap_or(default);
    let mut message = keybinds::display_name(binding);
    if enabled && options.selected_native_key == Some((index, 0)) {
        message = Text::literal("> ")
            .append(message.with_color(text::WHITE))
            .append(" <")
            .with_color(text::YELLOW);
    }

    let hud = button(0.0, 0.0, 310.0, Text::literal(hud_label.clone()), Act::NativeButton(index, 0));
    let crouch = button(0.0, 0.0, 150.0, Text::literal(crouch_label.clone()), Act::NativeButton(index, 1));
    let key = if enabled {
        button(0.0, 0.0, 150.0, message, Act::NativeKey(index, 0))
    } else {
        inactive(button(0.0, 0.0, 150.0, message, Act::None))
    };
    core::ops::ControlFlow::Break(vec![
        Row::Note(entry.title.clone()),
        Row::Buttons(hud, None),
        Row::Buttons(crouch, Some(key)),
    ])
}
