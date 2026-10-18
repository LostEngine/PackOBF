# How to test a resource pack
PackOBF comes with a Python script that lets you launch a Minecraft client that loads your resource pack and prints 
the logs in the console. You can use it using this command:
```bash
python mc-test-resource-pack/script.py <path-to-your-resource-pack> --version <version>
```

The script will automatically close the Minecraft client 3 seconds after the latest log message. 
You can keep it open by using the `--keep-open` flag.

> [!WARNING]
> You must have Python 3.10+ installed to use this script.
