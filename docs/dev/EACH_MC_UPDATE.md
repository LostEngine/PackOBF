# What to do each Minecraft update
- Update files in [`/packobf/src/minecraft/`](/packobf/src/minecraft/)
- Update each file in [`/packobf/src/resource_pack/files/`](/packobf/src/resource_pack/files/). 
  Websites that can be useful:
  - https://github.com/SpyglassMC/vanilla-mcdoc/tree/main/java/assets Mcdoc specifies which versions add/remove fields
  - https://misode.github.io/generators/ Uses Mcdoc with a Web UI
  - https://minecraft.wiki/
  - Minecraft release notes
- Update [`/packobf/src/version.rs`](/packobf/src/version.rs)
- Update Minecraft versions in [`packobf_gui/qml/main.qml`](/packobf_gui/qml/main.qml) and [`packobf_gui/src/cxxqt_object.rs`](/packobf_gui/src/cxxqt_object.rs)
