plugins {
    id("java")
    id("maven-publish")
    id("com.vanniktech.maven.publish") version "0.37.0"
}

group = "me.misieur"
version = "0.3.0-beta.1"

java {
    toolchain {
        languageVersion = JavaLanguageVersion.of(21)
    }
    withJavadocJar()
    withSourcesJar()
}

publishing {
    publications {
        create<MavenPublication>("mavenJava") {
            artifactId = "packobf"
            from(components["java"])
        }
    }
}

mavenPublishing {
    publishToMavenCentral()

    signAllPublications()

    coordinates(group.toString(), "packobf", version.toString())

    pom {
        name = "PackOBF"
        description = "An open-source Minecraft: Java Edition resource pack minimizer written in Rust."
        inceptionYear = "2026"
        url = "https://github.com/LostEngine/PackOBF/"
        licenses {
            license {
                name = "The MIT License"
                url = "https://github.com/LostEngine/PackOBF/blob/main/LICENSE.md"
                distribution = "repo"
            }
        }
        developers {
            developer {
                id = "misieur"
                name = "Misieur"
                url = "https://github.com/misieur/"
            }
        }
        scm {
            url = "https://github.com/LostEngine/PackOBF/"
            connection = "scm:git:git://github.com/LostEngine/PackOBF.git"
            developerConnection = "scm:git:ssh://git@github.com/LostEngine/PackOBF.git"
        }
    }
}


val nativeDir = layout.buildDirectory.dir("external-natives")

tasks.processResources {
    from(nativeDir) {
        into("packobf-natives")
    }
    from("../LICENSE.md")
}

tasks.named("publish") {
    dependsOn("processResources")
}
