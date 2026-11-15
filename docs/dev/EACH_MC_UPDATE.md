# What to do each Minecraft update
- Update files in [`/packobf/src/minecraft/`](/packobf/src/minecraft/) files in the `json` folder need to be updated using the mcdoc2rust crate, make sure to keep `crate::resource_pack::identifier::TextureIdWithExt` and the custom Deserializer for Sound
- Update [`/packobf/src/version.rs`](/packobf/src/version.rs)
- Update Minecraft versions in the [Linux](/packobf_gui/platforms/linux/options.rs), [macOS](/packobf_gui/platforms/macos/Sources/PackView.swift), and [Windows](/packobf_gui/platforms/windows/MainWindow.xaml.cs) GUI options
- Update [`dev.misieur.packobf.options.MinecraftVersion`](/java/src/main/java/dev/misieur/packobf/options/MinecraftVersion.java) version enum
- Update [`README.md`](/README.md)
