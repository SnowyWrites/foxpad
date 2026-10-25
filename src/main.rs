#![windows_subsystem = "windows"]
use std::{cell::Cell, rc::Rc};
use fltk::{app, browser::HoldBrowser, button::{Button, CheckButton}, dialog, draw, enums::*, frame::Frame, group::Group, image::PngImage, input::{Input, IntInput}, prelude::*, text::*, window::Window};

#[derive(Clone, Copy)]
enum Msg { New, Open, Save, SaveAs, Quit, Undo, Cut, Copy, Paste, SelectAll, TimeDate, Find, FindNext, ReplaceOne, ReplaceAll, WordWrap, Font, ApplyFont, About }

fn pick(kind: dialog::NativeFileChooserType) -> Option<String> {
    let mut c = dialog::NativeFileChooser::new(kind);
    c.show();
    let p = c.filename().to_string_lossy().to_string();
    if p.is_empty() { None } else { Some(p) }
}

//find/replace dialong helper
fn make_find_dialog(s: app::Sender<Msg>) -> (Window, Input, Input, CheckButton) {
    let win = Window::default().with_size(380, 135).with_label("Find and Replace");
    let find_in = Input::new(100, 10, 170, 24, "Find what:");
    let repl_in = Input::new(100, 40, 170, 24, "Replace with:");
    let case_btn = CheckButton::new(10, 75, 150, 24, "Match case");
    let mut b_next = Button::new(280, 10, 90, 24, "Find Next");
    let mut b_repl = Button::new(280, 40, 90, 24, "Replace");
    let mut b_all = Button::new(280, 70, 90, 24, "Replace All");
    let mut b_close = Button::new(280, 100, 90, 24, "Close");
    b_next.set_callback(move |_| s.send(Msg::FindNext));
    b_repl.set_callback(move |_| s.send(Msg::ReplaceOne));
    b_all.set_callback(move |_| s.send(Msg::ReplaceAll));
    let mut w = win.clone();
    b_close.set_callback(move |_| w.hide());
    win.end();
    (win, find_in, repl_in, case_btn)
}

// selects the next match after the cursor, wrapping to the top. false = no match
fn find_next(buf: &mut TextBuffer, ed: &mut TextEditor, needle: &str, match_case: bool) -> bool {
    if needle.is_empty() {
        return false;
    }
    let start = match buf.selection_position() {
        Some((_, end)) => end,
        None => ed.insert_position(),
    };
    let hit = buf
    .search_forward(start, needle, match_case)
    .or_else(|| buf.search_forward(0, needle, match_case));
    if let Some(pos) = hit {
        let end = pos + needle.len() as i32;
        buf.select(pos, end);
        ed.set_insert_position(end);
        ed.show_insert_position();
        true
    } else {
        false
    }
}

fn hide_all(menus: &mut [Group]) {
    for m in menus.iter_mut() {
        m.hide();
    }
}

fn any_open(menus: &[Group]) -> bool {
    menus.iter().any(|m| m.visible())
}

// opens menu `which`, closing the others; clicking an open menu's button closes it
fn toggle_menu(menus: &mut [Group], which: usize) {
    let was_open = menus[which].visible();
    hide_all(menus);
    if !was_open {
        menus[which].show();
    }
}

fn make_font_dialog(s: app::Sender<Msg>) -> (Window, HoldBrowser, IntInput) {
    let win = Window::default().with_size(300, 290).with_label("Font");
    let list = HoldBrowser::new(10, 25, 190, 250, "Font:").with_align(Align::TopLeft);
    let mut size_in = IntInput::new(210, 25, 80, 24, "Size:").with_align(Align::TopLeft);
    size_in.set_value("14");
    let mut b_apply = Button::new(210, 70, 80, 24, "Apply");
    let mut b_close = Button::new(210, 100, 80, 24, "Close");
    b_apply.set_callback(move |_| s.send(Msg::ApplyFont));
    let mut w = win.clone();
    b_close.set_callback(move |_| w.hide());
    win.end();
    (win, list, size_in)
}

//About dialog box
fn make_about_dialog() -> Window {
    let win = Window::default().with_size(320, 170).with_label("About FoxPad");
    let mut icon = Frame::new(15, 15, 64, 64, "");
    if let Ok(mut img) = PngImage::from_data(include_bytes!("../assets/foxpad.png")) {
        img.scale(64, 64, true, true);
        icon.set_image(Some(img));
    }
    let mut text = Frame::new(95, 15, 210, 100, "");
    text.set_label(&format!(
        "FoxPad {}\n\nA tiny retro notepad,\nbuilt with Rust and FLTK.\n\nMade by SnowyFoxPhantom",
        env!("CARGO_PKG_VERSION")
    ));
    text.set_align(Align::Left | Align::Inside | Align::Top);
    let mut ok = Button::new(120, 130, 80, 26, "OK");
    let mut w = win.clone();
    ok.set_callback(move |_| w.hide());
    win.end();
    win
}

//menu dropdown
fn make_dropdown(x: i32, items: &[(&str, &str, Msg)], s: app::Sender<Msg>) -> Group {
    let mut drop = Group::new(x, 25, 200, items.len() as i32 * 21 + 4, "");
    drop.set_frame(FrameType::UpBox);
    for (i, (label, hint, msg)) in items.iter().enumerate() {
        let mut b = Button::new(x + 2, 27 + i as i32 * 21, 196, 21, *label);
        b.set_frame(FrameType::FlatBox);
        b.set_align(Align::Left | Align::Inside);

        let hint = hint.to_string();
        b.draw(move |btn| {
            draw::draw_box(FrameType::FlatBox, btn.x(), btn.y(), btn.w(), btn.h(), btn.color());
            draw::set_draw_color(btn.label_color());
            draw::set_font(btn.label_font(), btn.label_size());
            draw::draw_text2(&btn.label(), btn.x() + 4, btn.y(), btn.w() - 8, btn.h(), Align::Left);
            draw::draw_text2(&hint, btn.x() + 4, btn.y(), btn.w() - 8, btn.h(), Align::Right);
        });

        b.handle(|btn, ev| match ev {
            Event::Enter => {
                btn.set_color(Color::from_rgb(0, 0, 128));
                btn.set_label_color(Color::White);
                btn.redraw();
                true
            }
            Event::Leave => {
                btn.set_color(Color::from_rgb(192, 192, 192));
                btn.set_label_color(Color::Black);
                btn.redraw();
                true
            }
            _ => false,
        });
        let (m, mut d) = (*msg, drop.clone());
        b.set_callback(move |btn| {
            btn.set_color(Color::from_rgb(192, 192, 192));
            btn.set_label_color(Color::Black);
            d.hide();
            s.send(m);
        });
    }
    drop.end();
    drop.hide();
    drop
}

//icon config for KDE Wayland
#[cfg(target_os = "linux")]
fn install_desktop_entry() {
    use std::{env, fs, path::PathBuf};

    let Ok(home) = env::var("HOME") else { return };
    let exe = match env::var("APPIMAGE") {
        Ok(p) => PathBuf::from(p),
        Err(_) => match env::current_exe() {
            Ok(p) => p,
            Err(_) => return,
        },
    };
    let data = PathBuf::from(&home).join(".local/share");
    let icon_path = data.join("icons/foxpad.png");
    let desktop_path = data.join("applications/foxpad.desktop");

    let _ = fs::create_dir_all(icon_path.parent().unwrap());
    let _ = fs::create_dir_all(desktop_path.parent().unwrap());
    let _ = fs::write(&icon_path, include_bytes!("../assets/foxpad.png"));

    let entry = format!(
        "[Desktop Entry]\nType=Application\nName=FoxPad\nExec=\"{}\"\nIcon={}\nStartupWMClass=foxpad\nCategories=Utility;TextEditor;\n",
        exe.display(),
                        icon_path.display()
    );
    // only rewrite if it changed, so we aren't touching the disk every launch
    if fs::read_to_string(&desktop_path).ok().as_deref() != Some(entry.as_str()) {
        let _ = fs::write(&desktop_path, entry);
    }
}

// unsaved changes helper
fn do_save(
    buf: &mut TextBuffer,
    wind: &mut Window,
    path: &mut Option<String>,
    dirty: &Cell<bool>,
    save_as: bool,
) -> bool {
    let target = match (&*path, save_as) {
        (Some(p), false) => Some(p.clone()),
        _ => pick(dialog::NativeFileChooserType::BrowseSaveFile),
    };
    if let Some(p) = target {
        match buf.save_file(&p) {
            Ok(_) => {
                wind.set_label(&format!("{} - FoxPad", p));
                *path = Some(p);
                dirty.set(false);
                return true;
            }
            Err(e) => dialog::alert_default(&format!("Could not save file:\n{}", e)),
        }
    }
    false
}

// true = fine to continue (New/Open/Exit), false = user cancelled
fn confirm_discard(
    buf: &mut TextBuffer,
    wind: &mut Window,
    path: &mut Option<String>,
    dirty: &Cell<bool>,
) -> bool {
    if !dirty.get() {
        return true;
    }
    match dialog::choice2_default("Save changes?", "Cancel", "Don't Save", "Save") {
        Some(2) => do_save(buf, wind, path, dirty, false),
        Some(1) => true,
        _ => false,
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    install_desktop_entry();
    let app = app::App::default().with_scheme(app::Scheme::Gtk);
    app::set_fonts("*");
    app::set_visual(Mode::Double).ok();
    app::background(192, 192, 192); // Win95 grey

    let (s, r) = app::channel::<Msg>();

    let mut wind = Window::default().with_size(640, 480).with_label("Untitled - FoxPad");

    let mut bar = Group::new(0, 0, 640, 25, "");
    bar.set_frame(FrameType::UpBox);
    let mut file_btn = Button::new(2, 2, 50, 21, "File");
    file_btn.set_frame(FrameType::FlatBox);
    let mut edit_btn = Button::new(54, 2, 50, 21, "Edit");
    edit_btn.set_frame(FrameType::FlatBox);
    let mut format_btn = Button::new(106, 2, 56, 21, "Format");
    format_btn.set_frame(FrameType::FlatBox);
    let mut help_btn = Button::new(164, 2, 44, 21, "Help");
    help_btn.set_frame(FrameType::FlatBox);
    bar.end();

    //unsaved changed flag
    let dirty = Rc::new(Cell::new(false));
    let mut buf = TextBuffer::default();
    {
        let dirty = dirty.clone();
        let mut mw = wind.clone();
        buf.add_modify_callback(move |_, ins, del, _, _| {
            if ins > 0 || del > 0 {
                dirty.set(true);
                let l = mw.label();
                if !l.starts_with('*') {
                    mw.set_label(&format!("*{}", l));
                }
            }
        });
    }

    let mut ed = TextEditor::new(0, 25, 640, 455, "");
    ed.set_buffer(buf.clone());
    ed.set_text_font(Font::Courier);
    ed.set_text_size(14);
    ed.set_cursor_style(fltk::text::Cursor::Simple);
    ed.set_cursor_color(Color::Black);
    ed.wrap_mode(WrapMode::AtBounds, 0);

    let file_drop = make_dropdown(2, &[
        ("New", "Ctrl+N", Msg::New),
                                  ("Open...", "Ctrl+O", Msg::Open),
                                  ("Save", "Ctrl+S", Msg::Save),
                                  ("Save As...", "", Msg::SaveAs),
                                  ("Exit", "Ctrl+Q", Msg::Quit),
    ], s);

    let edit_drop = make_dropdown(54, &[
        ("Undo", "Ctrl+Z", Msg::Undo),
                                  ("Cut", "Ctrl+X", Msg::Cut),
                                  ("Copy", "Ctrl+C", Msg::Copy),
                                  ("Paste", "Ctrl+V", Msg::Paste),
                                  ("Select All", "Ctrl+A", Msg::SelectAll),
                                  ("Find / Replace...", "Ctrl+F", Msg::Find),
                                  ("Find Next", "F3", Msg::FindNext),
                                  ("Time/Date", "F5", Msg::TimeDate),
    ], s);

    let format_drop = make_dropdown(106, &[
        ("✓ Word Wrap", "", Msg::WordWrap),
                                    ("   Font...", "", Msg::Font),
    ], s);

    let help_drop = make_dropdown(164, &[
        ("About FoxPad", "", Msg::About),
    ], s);

    let menus = vec![file_drop.clone(), edit_drop.clone(), format_drop.clone(), help_drop.clone()];

    let mut m = menus.clone();
    file_btn.set_callback(move |_| toggle_menu(&mut m, 0));
    let mut m = menus.clone();
    edit_btn.set_callback(move |_| toggle_menu(&mut m, 1));
    let mut m = menus.clone();
    format_btn.set_callback(move |_| toggle_menu(&mut m, 2));
    let mut m = menus.clone();
    help_btn.set_callback(move |_| toggle_menu(&mut m, 3));

    let mut m = menus.clone();
    ed.handle(move |_, ev| {
        if ev == Event::Push || (ev == Event::KeyDown && app::event_key() == Key::Escape) {
            hide_all(&mut m);
        }
        for menu in m.iter_mut() {
            if menu.visible() { menu.redraw(); } // keeps open menus on top
        }
        if ev == Event::KeyDown && app::event_key() == Key::from_i32(0xffc2) { // F5
            s.send(Msg::TimeDate);
            return true;
        }
        if ev == Event::KeyDown && app::event_key() == Key::from_i32(0xffc0) { // F3
            s.send(Msg::FindNext);
            return true;
        }
        if ev == Event::KeyDown && app::is_event_ctrl() {
            match app::event_key().to_char() {
                Some('n') => { s.send(Msg::New); return true; }
                Some('o') => { s.send(Msg::Open); return true; }
                Some('s') => { s.send(Msg::Save); return true; }
                Some('q') => { s.send(Msg::Quit); return true; }
                Some('f') | Some('h') => { s.send(Msg::Find); return true; }
                _ => {}
            }
        }
        false
    });

    let mut m = menus.clone();
    wind.handle(move |_, ev| {
        if ev == Event::Unfocus && any_open(&m) {
            hide_all(&mut m);
        }
        if (ev == Event::KeyDown || ev == Event::Shortcut)
            && app::event_key() == Key::Escape
            && any_open(&m)
            {
                hide_all(&mut m);
                return true;
            }
            false
    });


    wind.resize_callback(move |_, _, _, w, _| bar.set_size(w, 25));

    wind.set_callback(move |_| {
        if app::event() == Event::Close {
            s.send(Msg::Quit);
        }
    });
    wind.resizable(&ed);
    wind.end();
    //app icon
    if let Ok(icon) = PngImage::from_data(include_bytes!("../assets/foxpad.png")) {
        wind.set_icon(Some(icon));
    }
    wind.set_xclass("foxpad");
    wind.show();

    let (mut find_win, mut find_in, repl_in, case_btn) = make_find_dialog(s);
    let (mut font_win, mut font_list, size_in) = make_font_dialog(s);
    let mut about_win = make_about_dialog();
    let mut wrap_on = true;
    let mut blink_ed = ed.clone();
    let blink_menus = menus.clone();
    let mut cursor_on = true;
    app::add_timeout3(0.5, move |handle| {
        if !any_open(&blink_menus) {
            cursor_on = !cursor_on;
            blink_ed.show_cursor(cursor_on);
        }
        app::repeat_timeout3(0.5, handle);
    });

    let mut path: Option<String> = None;

    while app.wait() {
        if let Some(msg) = r.recv() {
            match msg {
                Msg::New => {
                    if confirm_discard(&mut buf, &mut wind, &mut path, &dirty) {
                        buf.set_text("");
                        path = None;
                        dirty.set(false);
                        wind.set_label("Untitled - FoxPad");
                    }
                }
                Msg::Open => {
                    if confirm_discard(&mut buf, &mut wind, &mut path, &dirty) {
                        if let Some(p) = pick(dialog::NativeFileChooserType::BrowseFile) {
                            match buf.load_file(&p) {
                                Ok(_) => {
                                    dirty.set(false);
                                    wind.set_label(&format!("{} - FoxPad", p));
                                    path = Some(p);

                                    // .LOG easter egg
                                    if buf.text_range(0, 4).as_deref() == Some(".LOG") {
                                        let stamp = chrono::Local::now().format("%-I:%M %p %-m/%-d/%Y").to_string();
                                        buf.append(&format!("\n{}\n", stamp));
                                        ed.set_insert_position(buf.length());
                                        ed.show_insert_position();
                                        ed.take_focus().ok();
                                    }
                                }
                                Err(e) => dialog::alert_default(&format!("Could not open file:\n{}", e)),
                            }

                        }
                    }
                }

                Msg::Save => {
                    do_save(&mut buf, &mut wind, &mut path, &dirty, false);
                }
                Msg::SaveAs => {
                    do_save(&mut buf, &mut wind, &mut path, &dirty, true);
                }
                Msg::Quit => {
                    if confirm_discard(&mut buf, &mut wind, &mut path, &dirty) {
                        app.quit();
                    }
                }
                Msg::Undo => { ed.undo(); ed.take_focus().ok(); }
                Msg::Cut => { ed.cut(); ed.take_focus().ok(); }
                Msg::Copy => { ed.copy(); ed.take_focus().ok(); }
                Msg::Paste => { ed.paste(); ed.take_focus().ok(); }
                Msg::SelectAll => {
                    buf.select(0, buf.length());
                    ed.take_focus().ok();
                }
                Msg::TimeDate => {
                    let stamp = chrono::Local::now().format("%-I:%M %p %-m/%-d/%Y").to_string();
                    let pos = ed.insert_position();
                    buf.insert(pos, &stamp);
                    ed.set_insert_position(pos + stamp.len() as i32);
                    ed.take_focus().ok();
                }
                Msg::Find => {
                    find_win.show();
                    find_in.take_focus().ok();
                }
                Msg::FindNext => {
                    let needle = find_in.value();
                    if needle.is_empty() {
                        find_win.show();
                    } else if !find_next(&mut buf, &mut ed, &needle, case_btn.is_checked()) {
                        dialog::message_default(&format!("Cannot find \"{}\"", needle));
                    }
                }
                Msg::ReplaceOne => {
                    let needle = find_in.value();
                    if let Some((start, _)) = buf.selection_position() {
                        let sel = buf.selection_text();
                        let same = if case_btn.is_checked() {
                            sel == needle
                        } else {
                            sel.to_lowercase() == needle.to_lowercase()
                        };
                        if same {
                            let repl = repl_in.value();
                            buf.replace_selection(&repl);
                            ed.set_insert_position(start + repl.len() as i32);
                        }
                    }
                    if !find_next(&mut buf, &mut ed, &needle, case_btn.is_checked()) {
                        dialog::message_default(&format!("No more instances of \"{}\" found.", needle));
                    }
                }
                Msg::ReplaceAll => {
                    let needle = find_in.value();
                    let repl = repl_in.value();
                    if !needle.is_empty() {
                        let mut pos = 0;
                        let mut count = 0;
                        while let Some(hit) = buf.search_forward(pos, &needle, case_btn.is_checked()) {
                            buf.replace(hit, hit + needle.len() as i32, &repl);
                            pos = hit + repl.len() as i32;
                            count += 1;
                        }
                        dialog::message_default(&format!("Replaced {} occurrence(s).", count));
                    }
                }
                Msg::WordWrap => {
                    wrap_on = !wrap_on;
                    ed.wrap_mode(if wrap_on { WrapMode::AtBounds } else { WrapMode::None }, 0);
                    if let Some(mut item) = format_drop.child(0) {
                        item.set_label(if wrap_on { "✓ Word Wrap" } else { "   Word Wrap" });
                    }
                    ed.take_focus().ok();
                }
                Msg::Font => {
                    if font_list.size() == 0 {
                        for name in app::get_font_names() {
                            font_list.add(&name);
                        }
                    }
                    font_win.show();
                }
                Msg::ApplyFont => {
                    let sel = font_list.value();
                    if sel > 0 {
                        if let Some(name) = font_list.text(sel) {
                            let size = size_in.value().parse::<i32>().unwrap_or(14).clamp(6, 72);
                            let slot = Font::Zapfdingbats;
                            Font::set_font(slot, &name);
                            ed.set_text_font(slot);
                            ed.set_text_size(size);
                            ed.redraw();
                        }
                    }
                }
                Msg::About => about_win.show(),
            }
        }
    }
}
