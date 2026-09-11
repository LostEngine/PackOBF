# How to use the library (for developers)

> [!NOTE]
> You may want to contact me if you want a collaboration to add features specific to your product.

### Java

Adding the dependency (Gradle)
###### build.gradle.kts
```kts
repositories {
    maven("https://repo.misieur.me/repository")
}

dependencies {
    compileOnly("dev.misieur:packobf:0.3.0")
}
```

Using PackOBF
```java
...
import dev.misieur.packobf.PackOBF;
import dev.misieur.packobf.options.Compression;
import dev.misieur.packobf.options.Options;
import dev.misieur.packobf.options.ShaderCompression;
import dev.misieur.packobf.progress.*;

import java.io.IOException;
import java.nio.file.Path;
import java.util.Locale;
import java.util.Optional;
...
    byte[] bytes = /* The byte array of your built resource pack readable by any software */;
    try {
        byte[] output = PackOBF.optimizeZip( // Optimize resource pack and returns the new byte array
                bytes,
                new Options( // Configure PackOBF
                        Compression.NORMAL,
                        ShaderCompression.NONE,
                        true,
                        true,
                        true,
                        Optional.empty(),
                        Optional.empty()
                ),
                (level, message) -> System.out.println(level.name().toUpperCase(Locale.ROOT) + ": " + message), // Message logger
                progress -> { // Progress logger (can be used in bossbar, for example)
                    switch (progress) {
                        case IdleProgress p -> System.out.println("Initializing...");
                        case ReadingZipProgress p -> System.out.println("Reading resource pack... " + p.current() + "/" + p.total());
                ...
                    }
                },
                Path.of("path/to/cachefile.bin") // Nullable
        );
    } catch (IOException e) { // PackOBF will throw a Java exception if it fails to optimize the resource pack
        e.printStackTrace();
    }
```