This folder contains all the assets' paths from `models`, `sounds` and `textures` 
folder that have existed in 1.21.1, 1.21.2, 1.21.4, 1.21.5, 1.21.6, 1.21.7, 1.21.9, 1.21.11, 26.1 and 26.2.

To build these files, you first need to download an asset folder (from [mcasset](https://github.com/InventivetalentDev/minecraft-assets))
then you can use this command to create a file in one asset folder:
```bash
find . -type f ! -name "_*" ! -name "*.txt" | sed 's|^\./||' | sort > files.txt
```

To combine multiple asset lists from different versions, use:
```bash
cat version1.txt version2.txt | sort -u > merged.txt
```
