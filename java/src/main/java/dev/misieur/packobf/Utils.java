package dev.misieur.packobf;

import java.io.FileNotFoundException;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;

public class Utils {
    public static Path extractFile(String resourcePath, Path targetDir) throws IOException {
        Files.createDirectories(targetDir);

        String fileName = Paths.get(resourcePath).getFileName().toString();
        Path targetFile = targetDir.resolve(fileName);

        if (Files.exists(targetFile)) return targetFile;

        try (InputStream in = StackWalker.getInstance(StackWalker.Option.RETAIN_CLASS_REFERENCE)
                .getCallerClass()
                .getResourceAsStream(resourcePath)) {

            if (in == null) {
                throw new FileNotFoundException(resourcePath);
            }

            Files.copy(in, targetFile);
        }

        return targetFile;
    }
}
