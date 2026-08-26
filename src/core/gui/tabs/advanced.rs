use crate::core::gui::config::ConfigEditor;
use crate::core::gui::utils::*;
use crate::core::gui::Gui;
use crate::core::gui::dialogs::SimpleOkDialog;
use crate::core::hachimi;
use egui_material3::{MaterialNumberField, MaterialSelect, MaterialTextField, SelectVariant};
use rust_i18n::t;
use std::thread;


#[allow(unused_variables)]
pub fn render(editor: &ConfigEditor, config: &mut crate::core::hachimi::Config, ui: &mut egui::Ui) {

    // ── Advanced ──────────────────────────────────────────────────────────────
    section_heading(ui, t!("config_editor.advanced_settings_heading"));

    ConfigEditor::list_tile_switch(ui, t!("config_editor.enable_file_logging"), &mut config.enable_file_logging, true);
    ConfigEditor::list_tile_switch(ui, t!("config_editor.enable_ipc"),           &mut config.enable_ipc,           true);
    ConfigEditor::list_tile_switch(ui, t!("config_editor.ipc_listen_all"),        &mut config.ipc_listen_all,       true);
    ConfigEditor::list_tile_switch(ui, t!("config_editor.ipv4_only"),             &mut config.ipv4_only,            true);

    if !ConfigEditor::row_filtered(&t!("config_editor.meta_index_url")) {
        ConfigEditor::maybe_draw_category_header(ui);
        ui.add(egui::Label::new(t!("config_editor.meta_index_url")).wrap());
        ui.set_max_width(ui.available_width());
        let res = ui.add(MaterialTextField::filled(&mut config.meta_index_url).lock_focus(true));
        if res.lost_focus() && config.meta_index_url.trim().is_empty() {
            config.meta_index_url = hachimi::Config::default().meta_index_url;
        }
        #[cfg(target_os = "windows")]
        if res.has_focus() {
            ui.memory_mut(|mem| mem.set_focus_lock_filter(res.id, egui::EventFilter {
                tab: true, horizontal_arrows: true, vertical_arrows: true, escape: true,
                ..Default::default()
            }));
        }
        ui.add_space(4.0);
    }

    if !ConfigEditor::row_filtered(&t!("config_editor.localized_data_dir")) {
        ConfigEditor::maybe_draw_category_header(ui);
        let avail_w = ui.available_width();
        ui.add(egui::Label::new(t!("config_editor.localized_data_dir")).wrap());
        let mut current_dir = config.localized_data_dir.clone()
            .unwrap_or_else(|| "localized_data".to_string());
        let mut sel = editor.localized_data_dirs.iter().position(|v| v == &current_dir);
        let mut select = MaterialSelect::new(&mut sel)
            .variant(SelectVariant::Outlined)
            .placeholder(&current_dir)
            .width(avail_w)
            .small();
        for (i, label) in editor.localized_data_dirs.iter().enumerate() {
            select = select.option(i, label);
        }
        if ui.add(select).changed() {
            if let Some(i) = sel { current_dir = editor.localized_data_dirs[i].clone(); }
        }
        config.localized_data_dir = Some(current_dir);
        ui.add_space(4.0);
    }

    ConfigEditor::list_tile_switch(ui, t!("config_editor.translator_mode"),             &mut config.translator_mode,             true);
    ConfigEditor::list_tile_switch(ui, t!("config_editor.apply_atlas_workaround"),       &mut config.apply_atlas_workaround,       true);
    ConfigEditor::list_tile_switch(ui, t!("config_editor.disable_outdated_asset_notif"), &mut config.disable_outdated_asset_notif, true);
    ConfigEditor::list_tile_switch(ui, t!("config_editor.replace_to_builtin_font"),      &mut config.replace_to_builtin_font,      true);
    ConfigEditor::list_tile_switch(ui, t!("config_editor.skip_first_time_setup"),        &mut config.skip_first_time_setup,        true);
    ConfigEditor::list_tile_switch(ui, t!("config_editor.lazy_translation_updates"),     &mut config.lazy_translation_updates,     true);
    ConfigEditor::list_tile_switch(ui, t!("config_editor.disable_auto_update_check"),    &mut config.disable_auto_update_check,    true);

    ConfigEditor::list_tile_switch(ui, t!("config_editor.disable_mod_downloads"), &mut config.disable_mod_downloads, true);

    ConfigEditor::list_tile_combo(ui, t!("config_editor.bg_update_mode"), "bg_update_mode",
        &mut config.bg_update_mode, &[
            (hachimi::BgUpdateMode::Disabled, &t!("disabled")),
            (hachimi::BgUpdateMode::Periodic, &t!("config_editor.bg_update_periodic")),
            (hachimi::BgUpdateMode::Silent,   &t!("config_editor.bg_update_silent")),
        ]);

    if config.bg_update_mode != hachimi::BgUpdateMode::Disabled
        && !ConfigEditor::row_filtered(&t!("config_editor.bg_update_interval")) {
        ConfigEditor::maybe_draw_category_header(ui);
        let mut minutes = (config.bg_update_interval_sec / 60) as i32;
        let prev_minutes = minutes;

        ui.add(egui::Label::new(t!("config_editor.bg_update_interval")).wrap());
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            let scale = crate::core::gui::utils::get_scale(ui.ctx());
            let number_w = 48.0 * scale;
            let (rect, _) = ui.allocate_exact_size(egui::vec2(number_w, 32.0), egui::Sense::hover());
            ui.put(
                rect,
                MaterialNumberField::filled(&mut minutes)
                    .range(1..=999)
                    .decimals(0),
            );
            ui.label(t!("config_editor.bg_update_interval_unit"));
        });
        ui.add_space(4.0);

        if minutes != prev_minutes {
            config.bg_update_interval_sec = (minutes as u64) * 60;
        }
    }

    ConfigEditor::list_tile_switch(ui, t!("config_editor.disable_translations"), &mut config.disable_translations, true);

    #[cfg(target_os = "android")]
    {
        ConfigEditor::list_tile_switch(ui, t!("config_editor.hook_libc_dlopen"), &mut config.android.hook_libc_dlopen, true);
        ConfigEditor::list_tile_switch(ui, t!("config_editor.keep_screen_on"),   &mut config.android.keep_screen_on,   true);
    }

    #[cfg(target_os = "windows")]
    {
        let supports_smtc    = crate::windows::capabilities::supports_smtc();
        let supports_toasts  = crate::windows::capabilities::supports_scheduled_toasts();
        let supports_taskbar = crate::windows::capabilities::supports_taskbar_progress();

        ConfigEditor::list_tile_switch(ui, t!("config_editor.discord_rpc"), &mut config.windows.discord_rpc, true);

        ConfigEditor::list_tile_switch_with_hint(
            ui, t!("config_editor.enable_smtc"), &mut config.windows.enable_smtc,
            supports_smtc, t!("config_editor.unavailable_wine_proton"),
        );

        for (label_key, val) in [
            ("config_editor.notification_tp",    &mut config.notification_tp    as &mut bool),
            ("config_editor.notification_rp",    &mut config.notification_rp),
            ("config_editor.notification_jobs",  &mut config.notification_jobs),
        ] {
            ConfigEditor::list_tile_switch_with_hint(
                ui, t!(label_key), val,
                supports_toasts, t!("config_editor.unavailable_wine_proton"),
            );
        }

        for (label_key, val) in [
            ("config_editor.taskbar_show_progress_on_download",      &mut config.windows.taskbar_show_progress_on_download   as &mut bool),
            ("config_editor.taskbar_show_progress_on_connecting",    &mut config.windows.taskbar_show_progress_on_connecting),
            ("config_editor.taskbar_show_progress_on_schedule_book", &mut config.windows.taskbar_show_progress_on_schedule_book),
        ] {
            ConfigEditor::list_tile_switch_with_hint(
                ui, t!(label_key), val,
                supports_taskbar, t!("config_editor.unavailable_wine_proton"),
            );
        }

        ConfigEditor::list_tile_text_field(ui, t!("config_editor.custom_title_name"), {
            let _ = config.custom_title_name.get_or_insert_with(String::new);
            config.custom_title_name.as_mut().unwrap()
        });
        if config.custom_title_name.as_deref() == Some("") {
            config.custom_title_name = None;
        }
    }

    ConfigEditor::list_tile_switch(ui, t!("config_editor.hide_now_loading"), &mut config.hide_now_loading, true);

    section_heading(ui, t!("config_editor.experimental_settings_heading"));

    if ConfigEditor::list_tile_switch(ui, t!("config_editor.auto_translate_stories"), &mut config.auto_translate_stories, true)
        .clicked() && config.auto_translate_stories
    {
        thread::spawn(|| {
            Gui::instance().unwrap().lock().unwrap_or_else(|e| e.into_inner())
                .show_window(Box::new(SimpleOkDialog::new(
                    &t!("warning"), &t!("config_editor.auto_tl_warning"), || {},
                )));
        });
    }

    if config.auto_translate_stories {
        {
            let mut url = config.sugoi_url.clone().unwrap_or_default();
            if ConfigEditor::list_tile_text_field(ui, t!("config_editor.sugoi_url"), &mut url).changed() {
                config.sugoi_url = if url.is_empty() { None } else { Some(url) };
            }
        }

        if ConfigEditor::list_tile_switch_described_danger(ui, t!("config_editor.auto_translate_ui"), &mut config.auto_translate_localize, true, t!("config_editor.auto_translate_ui_desc"))
            .clicked() && config.auto_translate_localize
        {
            thread::spawn(|| {
                Gui::instance().unwrap().lock().unwrap_or_else(|e| e.into_inner())
                    .show_window(Box::new(SimpleOkDialog::new(
                        &t!("warning"), &t!("config_editor.auto_tl_warning"), || {},
                    )));
            });
        }
    }

    ConfigEditor::list_tile_switch(ui, t!("config_editor.unlock_live_chara"),  &mut config.unlock_live_chara,  true);
    ConfigEditor::list_tile_switch(ui, t!("config_editor.msgpack_notifier"),   &mut config.msgpack_notifier,   true);

    if config.msgpack_notifier {
        ConfigEditor::list_tile_text_field(ui, t!("config_editor.msgpack_notifier_host"), &mut config.msgpack_notifier_host);
        ConfigEditor::list_tile_switch(ui, t!("config_editor.msgpack_notifier_request"), &mut config.msgpack_notifier_request, true);
        ConfigEditor::list_tile_number(ui, t!("config_editor.msgpack_notifier_connection_timeout_ms"),
            &mut config.msgpack_notifier_connection_timeout_ms, 100..=30000, 100.0, Some("ms"));
        ConfigEditor::list_tile_switch(ui, t!("config_editor.msgpack_notifier_print_error"), &mut config.msgpack_notifier_print_error, true);
    }

    ConfigEditor::list_tile_switch(ui, t!("config_editor.dump_msgpack"),         &mut config.dump_msgpack,         true);
    ConfigEditor::list_tile_switch(ui, t!("config_editor.dump_msgpack_request"), &mut config.dump_msgpack_request, true);

    section_heading(ui, t!("config_editor.developer_settings_heading"));

    ConfigEditor::list_tile_switch(ui, t!("config_editor.debug_mode"), &mut config.debug_mode, true);

    if ConfigEditor::list_tile_switch(ui, t!("config_editor.text_debug"), &mut config.text_debug, true)
        .clicked() && !config.text_debug
    {
        config.text_log = false; config.text_property_dump = false;
        config.text_localize_dump = false; config.text_position_debug = false;
        config.text_path_debug = false;
    }

    if config.text_debug {
        ConfigEditor::list_tile_switch(ui, format!("    - {}", t!("config_editor.text_log")),            &mut config.text_log,            true);
        ConfigEditor::list_tile_switch(ui, format!("    - {}", t!("config_editor.text_property_dump")),  &mut config.text_property_dump,  true);
        ConfigEditor::list_tile_switch(ui, format!("    - {}", t!("config_editor.text_localize_dump")),  &mut config.text_localize_dump,  true);
        ConfigEditor::list_tile_switch(ui, format!("    - {}", t!("config_editor.text_position_debug")), &mut config.text_position_debug, true);
        ConfigEditor::list_tile_switch(ui, format!("    - {}", t!("config_editor.text_path_debug")),     &mut config.text_path_debug,     true);
    }
}
