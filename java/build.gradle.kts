plugins {
    id("java")
    id("maven-publish")
}

group = "dev.misieur"
version = "0.3.0"

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
