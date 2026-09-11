# PackOBF Options

<!-- TOC -->
* [PackOBF Options](#packobf-options)
  * [Compression Level (`compression`)](#compression-level-compression)
  * [Shader Compression (`shader_compression`) (Experimental/Deprecated)](#shader-compression-shader_compression-experimentaldeprecated)
  * [Rename files (`rename_files`)](#rename-files-rename_files)
  * [Block unzipping (`block_unzipping`)](#block-unzipping-block_unzipping)
  * [Corrupt PNG images (`corrupt_png_files`)](#corrupt-png-images-corrupt_png_files)
  * [Target version (`target_version`)](#target-version-target_version)
  * [Advanced options](#advanced-options)
    * [Num Thread (`num_threads`)](#num-thread-num_threads)
<!-- TOC -->

## Compression Level (`compression`)

Used for any file in the resource pack and PNG images before being compressed in the resource pack.

| Compression level | Description                                                                                                                                 |
|-------------------|---------------------------------------------------------------------------------------------------------------------------------------------|
| Fastest           | Uses libdeflate level 6                                                                                                                     |
| Fast              | Uses libdeflate level 12                                                                                                                    |
| Normal            | Pre-processes data with libdeflate level 9 and based on the results chooses zopfli options that would give the best time/compression ratio. |
| Best              | Same as Normal but with higher zopfli compression.                                                                                          |
| Ultra             | Zopfli with 40 iteration_count, 40 iterations_without_improvement and 25 maximum_block_splits                                               |

> [!NOTE]
> You may not want to use the Ultra compression level if it's not for testing/benchmarking
> as it will take a long time to compress for a result close to Best.

## Shader Compression (`shader_compression`) (Experimental/Deprecated)

| Shader compression   | Description                                                                |
|----------------------|----------------------------------------------------------------------------|
| None                 | No compression                                                             |
| Minify               | Uses glsl_lang to parse and rewrite the shader without useless characters  |
| Minify and obfuscate | Same as Minify but replaces names of variables/functions with smaller ones |

## Rename files (`rename_files`)

Renames resource pack overlays, models, textures, and sounds to small names, with the most referenced files
having smaller names than the least referenced files. PackOBF creates mappings internally to rename
references in other files (e.g., textures in models).

> [!NOTE]
> PackOBF knows which files are vanilla files (e.g., assets/textures/item/apple.png) for all the versions supported by the
> PackOBF version that you are using, if you want to support newer versions, keep in mind to update PackOBF.

## Block unzipping (`block_unzipping`)

Adds additional information to the resource pack file to break tools wanting to write its files onto a file system.

## Corrupt PNG images (`corrupt_png_files`)

Replaces some parts of PNG images that Minecraft's image reader does not care about, but that would make common image
software think the image is broken.

> [!NOTE]
> This option actually allows removing four bytes from each PNG image, which will make your resource pack smaller.

## Target version (`target_version`)

If defined, removes files from overlays that are meant for other Minecraft versions, remove fields in JSON files that
do not exist on the chosen Minecraft version and the `items` folder for versions before 1.21.4.

---

## Advanced options

### Num Thread (`num_threads`)

The number of threads that PackOBF will use for concurrent tasks.