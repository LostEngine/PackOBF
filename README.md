# PackOBF,
### an open-source Minecraft: Java Edition resource pack minimizer written in Rust.

PackOBF is a resource pack minimizer and checker for Minecraft: Java Edition.
PackOBF is able to reduce the size of a resource pack and detect issues with files
like PNG images and JSON files (items, models, blockstates, fonts, sound definitions, atlas).
By parsing these files like Minecraft would do, it can detect issues like empty required fields,
wrong value types in JSON files and malformed PNG/OGG files.

## Usage
PackOBF comes with a command-line interface, here is how to use it:
```
Usage: packobf_cli [OPTIONS] --input-file <FILE> --output-file <FILE>

Options:
  -i, --input-file <FILE>                        
  -o, --output-file <FILE>                       
  -p, --preset <PRESET>                          [possible values: simplest, normal, max]
  -l, --log-level <LOG_LEVEL>                    [default: info] [possible values: info, warning, error, none]
      --cache-file <CACHE_FILE>                  
  -c, --compression <COMPRESSION>                [default: normal] [possible values: simplest, normal, max]
      --shader-compression <SHADER_COMPRESSION>  [default: minify] [possible values: none, minify, minify-and-obfuscate]
      --rename-files                             
      --block-unzipping                          
      --corrupt-png-files                        
  -h, --help                                     Print help
  -V, --version                                  Print version
```

> [!TIP]
> Using `--preset` is a good choice when trying PackOBF for the first time, possible values are: `simplest` (fastest), `normal` (balanced), `max` (slowest).

~~Alternatively, you can use PackOBF in your browser at https://packobf.misieur.me/ however, it will be way slower than the native app 
because of internet browser restrictions.~~ PackOBF on internet browsers is deprecated.

## List of features

### JSON Files
PackOBF will parse JSON files into memory and will know for JSON files in `items`, `models`, `blockstates`, `fonts`, `sound_definitions`, and `atlas` folders
which fields exist, which one are required, their value type (string, integer, etc.) and their default value.
With all of this data, PackOBF is able to remove fields that are unknown by Minecraft, values that are
already the default one, spaces and new lines.
> [!NOTE]
> For any other JSON file, PackOBF will only simplify them, remove spaces, newlines, `.0` for numbers, etc.

### PNG Files
PackOBF uses [oxipng](https://github.com/oxipng/oxipng) to minimize losslessly PNG files.  
Moreover, if you enable the *Corrupt PNG Files* option (`--corrupt-png-files`), your PNG images will be
corrupted and unreadable for a lof of image software but Minecraft and will remove some bytes from your PNG images.

### Resource pack file
PackOBF has its own way of writing ZIP files, which will not work with some popular ZIP file extractors,
but does work with Minecraft. This way of writing ZIP files is not conventional, but it allows saving file size
when duplicated files are in the resource pack and does not write useless data like folders and other fields on the ZIP file.
PackOBF by itself will break some file software however the `jar -xf output.zip` command from Java will still be able to unzip it.
By enabling the *Block resource pack from unzipping* option (`--block-unzipping`), it will totally block Java and a lot of software
(including JD-GUI, Minecraft mods) from unzipping your resource pack while Minecraft can still load it.

### Renaming files
PackOBF is able to rename resource pack overlays, models, textures and sounds that don't override Minecraft files to smaller names
(example: `assets/_/textures/i/a.png` instead of `assets/minecraft/textures/item/my_cool_item.png`) while keeping everything working.

### Core Shaders (Experimental)
PackOBF is able to parse core shaders, minify them, rename variables and functions to short names, it can be enabled using the
`--shader-compression [none, minify, minify-and-obfuscate]` option, `minify-and-obfuscate` will rename variables and functions
which might break your shaders, while `minify` does not.

## License
PackOBF is open-source software distributed under the MIT license. See [`LICENSE.md`](LICENSE.md) for complete license.

> [!NOTE]
> No code from any other project has been "stolen" or used as inspiration, all researches were made using public server resource packs and online resources such as [ImHex](https://github.com/werwolv/imhex).

## How to use the library (for developers)

See [How to use the library (for developers)](docs/usage/HOW_TO_USE_LIBRARY_DEVS.md)
