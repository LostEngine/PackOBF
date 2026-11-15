use adw::prelude::*;
use packobf_gui::app::{
    Compression, MinecraftVersion, Options, ShaderCompression, SoundQuality, default_options,
};

pub(super) struct Controls {
    compression: Choice<Compression>,
    shader: Choice<ShaderCompression>,
    sound: Choice<SoundQuality>,
    version: Choice<Option<MinecraftVersion>>,
    rename: adw::SwitchRow,
    block: adw::SwitchRow,
    corrupt: adw::SwitchRow,
    threads: gtk::SpinButton,
}

impl Controls {
    pub(super) fn new(page: &adw::PreferencesPage) -> Self {
        let defaults = default_options();
        let group = adw::PreferencesGroup::builder()
            .title("Optimization settings")
            .build();
        page.add(&group);

        let compression = Choice::new(
            &group,
            "Compression",
            "Global compression level for the resource pack.",
            &[("Fast", Compression::Fast), ("Normal", Compression::Normal)],
            1,
        );
        let shader = Choice::new(
            &group,
            "Shader compression (Experimental)",
            "Parses GLSL core shaders to minify and obfuscate them.",
            &[
                ("None", ShaderCompression::None),
                ("Minify", ShaderCompression::Minify),
                (
                    "Minify and obfuscate",
                    ShaderCompression::MinifyAndObfuscate,
                ),
            ],
            0,
        );
        let sound = Choice::new(
            &group,
            "Sound quality",
            "Re-encodes sound files using libvorbis with the requested quality.",
            &[
                ("Dynamic", SoundQuality::Dynamic),
                ("-1", SoundQuality::MinusOne),
                ("0", SoundQuality::Zero),
                ("1", SoundQuality::One),
                ("2", SoundQuality::Two),
                ("3", SoundQuality::Three),
                ("4", SoundQuality::Four),
                ("5", SoundQuality::Five),
                ("6", SoundQuality::Six),
                ("7", SoundQuality::Seven),
                ("8", SoundQuality::Eight),
                ("9", SoundQuality::Nine),
                ("10", SoundQuality::Ten),
            ],
            0,
        );
        let version = Choice::new(
            &group,
            "Target Minecraft version",
            "Removes parts of the resource pack specific to other Minecraft versions.",
            &[
                ("Any", None),
                ("1.21.1", Some(MinecraftVersion::V1_21_1)),
                ("1.21.2", Some(MinecraftVersion::V1_21_2)),
                ("1.21.4", Some(MinecraftVersion::V1_21_4)),
                ("1.21.5", Some(MinecraftVersion::V1_21_5)),
                ("1.21.6", Some(MinecraftVersion::V1_21_6)),
                ("1.21.7", Some(MinecraftVersion::V1_21_7)),
                ("1.21.9", Some(MinecraftVersion::V1_21_9)),
                ("1.21.11", Some(MinecraftVersion::V1_21_11)),
                ("26.1", Some(MinecraftVersion::V26_1)),
                ("26.2", Some(MinecraftVersion::V26_2)),
                ("26.3", Some(MinecraftVersion::V26_3)),
            ],
            0,
        );
        let rename = switch(
            &group,
            "Rename files",
            "Renames textures, models, and sounds to shorter names.",
            defaults.rename_files,
        );
        let block = switch(
            &group,
            "Block resource pack unzipping",
            "Adds some bytes to the resource pack to prevent files from being extracted on a file system.",
            defaults.block_unzipping,
        );
        let corrupt = switch(
            &group,
            "Corrupt PNG files",
            "Corrupts PNG files in a way that makes them unreadable for most software except for Minecraft.",
            defaults.corrupt_png_files,
        );

        let advanced = adw::ExpanderRow::builder()
            .title("Advanced options")
            .build();
        let advanced_group = adw::PreferencesGroup::new();
        advanced_group.add(&advanced);
        page.add(&advanced_group);

        let threads = gtk::SpinButton::with_range(0.0, 256.0, 1.0);
        threads.set_valign(gtk::Align::Center);
        threads.set_numeric(true);
        let threads_row = adw::ActionRow::builder()
            .title("Worker threads")
            .subtitle("Zero for automatic")
            .activatable_widget(&threads)
            .build();
        threads_row.add_suffix(&threads);
        advanced.add_row(&threads_row);

        Self {
            compression,
            shader,
            sound,
            version,
            rename,
            block,
            corrupt,
            threads,
        }
    }

    pub(super) fn options(&self) -> Options {
        self.threads.update();
        let threads = self.threads.value_as_int() as usize;
        Options {
            compression: self.compression.value(),
            shader_compression: self.shader.value(),
            sound_quality: self.sound.value(),
            target_version: self.version.value(),
            rename_files: self.rename.is_active(),
            block_unzipping: self.block.is_active(),
            corrupt_png_files: self.corrupt.is_active(),
            num_threads: (threads > 0).then_some(threads),
        }
    }

    pub(super) fn connect_changed(&self, changed: impl Fn() + Clone + 'static) {
        for row in [
            &self.compression.row,
            &self.shader.row,
            &self.sound.row,
            &self.version.row,
        ] {
            let changed = changed.clone();
            row.connect_selected_notify(move |_| changed());
        }
        for row in [&self.rename, &self.block, &self.corrupt] {
            let changed = changed.clone();
            row.connect_active_notify(move |_| changed());
        }
        self.threads.connect_changed(move |_| changed());
    }
}

struct Choice<T> {
    row: adw::ComboRow,
    options: Vec<T>,
}

impl<T: Clone> Choice<T> {
    fn new(
        group: &adw::PreferencesGroup,
        title: &str,
        subtitle: &str,
        choices: &[(&str, T)],
        selected: u32,
    ) -> Self {
        let labels: Vec<_> = choices.iter().map(|(label, _)| *label).collect();
        let row = adw::ComboRow::builder()
            .title(title)
            .subtitle(subtitle)
            .model(&gtk::StringList::new(&labels))
            .selected(selected)
            .build();
        group.add(&row);
        let options = choices.iter().map(|(_, option)| option.clone()).collect();
        Self { row, options }
    }

    fn value(&self) -> T {
        self.options[self.row.selected() as usize].clone()
    }
}

fn switch(
    group: &adw::PreferencesGroup,
    title: &str,
    subtitle: &str,
    active: bool,
) -> adw::SwitchRow {
    let row = adw::SwitchRow::builder()
        .title(title)
        .subtitle(subtitle)
        .active(active)
        .build();
    group.add(&row);
    row
}
