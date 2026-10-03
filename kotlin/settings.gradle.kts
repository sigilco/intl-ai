pluginManagement {
    repositories {
        // Google's mirror of Maven Central: reachable where repo1.maven.org is throttled.
        maven("https://maven-central.storage-download.googleapis.com/maven2/")
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositories {
        maven("https://maven-central.storage-download.googleapis.com/maven2/")
        google()
        mavenCentral()
    }
}

rootProject.name = "intl-ai-kotlin"
include(":intlai", ":intlai-android")
