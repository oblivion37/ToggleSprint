fn sprint_settings_widths(
    mut layout: (Vec<Widget>, f32, f32, f32),
    options: &OptionsUi,
    settings: &Settings,
) -> (Vec<Widget>, f32, f32, f32) {
    let Page::NativeMod(index) = options.page() else {
        return layout;
    };
    if !settings.mods.installed.get(index).is_some_and(|entry| entry.id == "togglesprint") {
        return layout;
    }

    let widgets = &mut layout.0;
    let pair = widgets.windows(2).position(|row| {
        matches!(row[0].kind, WKind::Button { act: Act::NativeButton(i, 1), .. } if i == index)
            && row[0].y == row[1].y
    });
    if let Some(i) = pair {
        // Keep the row's outside edges and gap when widening the toggle.
        let extra_width = 60.0;
        widgets[i].w += extra_width;
        widgets[i + 1].x += extra_width;
        widgets[i + 1].w -= extra_width;
    }
    layout
}
