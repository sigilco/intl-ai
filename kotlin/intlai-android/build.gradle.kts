import java.util.Properties

plugins {
    id("com.android.library")
    `maven-publish`
}

group = "io.github.sigilco"
version = providers.gradleProperty("intlAiVersion").get()

val workspaceRoot = rootDir.parentFile
val bindingsDir = File(workspaceRoot, "crates/intl-ai-uniffi/bindings")
val intlAiNdkVersion = "27.2.12479018"

val sdkDir: File = run {
    val props = Properties()
    val localProps = rootProject.file("local.properties")
    if (localProps.exists()) localProps.inputStream().use { props.load(it) }
    val dir = props.getProperty("sdk.dir")
        ?: System.getenv("ANDROID_SDK_ROOT")
        ?: System.getenv("ANDROID_HOME")
        ?: error("Android SDK not found: set sdk.dir in kotlin/local.properties or ANDROID_HOME")
    File(dir)
}
val ndkDir = File(sdkDir, "ndk/$intlAiNdkVersion")

val cargoNdkBuild = tasks.register<Exec>("cargoNdkBuild") {
    group = "build"
    description = "Compiles libintl_ai_uniffi for the Android ABIs via cargo-ndk."
    workingDir = workspaceRoot
    environment("ANDROID_NDK_HOME", ndkDir.absolutePath)
    environment("ANDROID_NDK_ROOT", ndkDir.absolutePath)
    commandLine(
        "cargo", "ndk",
        "-t", "armeabi-v7a",
        "-t", "arm64-v8a",
        "-t", "x86",
        "-t", "x86_64",
        "-P", "24",
        "-o", layout.buildDirectory.dir("jniLibs").get().asFile.absolutePath,
        "--manifest-path", File(workspaceRoot, "Cargo.toml").absolutePath,
        "build", "--release", "-p", "intl-ai-uniffi"
    )
}

tasks.register<Exec>("generateKotlinBindings") {
    group = "build"
    description = "Regenerates the UniFFI Kotlin bindings into crates/intl-ai-uniffi/bindings."
    workingDir = workspaceRoot
    commandLine("bash", "crates/intl-ai-uniffi/scripts/generate-bindings.sh")
}

android {
    namespace = "io.github.sigilco.intlai"
    compileSdk = 36
    ndkVersion = intlAiNdkVersion

    defaultConfig {
        minSdk = 24
    }

    sourceSets {
        named("main") {
            kotlin.directories.add(bindingsDir.absolutePath)
            jniLibs.directories.add(layout.buildDirectory.dir("jniLibs").get().asFile.absolutePath)
        }
    }

    publishing {
        singleVariant("release")
    }
}

dependencies {
    // The UniFFI Kotlin bindings call into Rust through JNA.
    implementation("net.java.dev.jna:jna:5.18.1@aar")
}

tasks.named("preBuild") { dependsOn(cargoNdkBuild) }

afterEvaluate {
    publishing {
        publications {
            create<MavenPublication>("release") {
                from(components["release"])
                pom {
                    name.set("intlai-android")
                    description.set("Android bindings for the intl-ai i18n engine (UniFFI)")
                    url.set("https://github.com/sigilco/intl-ai")
                    licenses {
                        license {
                            name.set("Apache-2.0")
                            url.set("https://www.apache.org/licenses/LICENSE-2.0.txt")
                        }
                    }
                    scm {
                        url.set("https://github.com/sigilco/intl-ai")
                    }
                }
            }
        }
    }
}
