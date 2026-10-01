plugins {
    kotlin("jvm")
    `maven-publish`
}

group = "io.github.sigilco"
version = providers.gradleProperty("intlAiVersion").get()

val workspaceRoot = rootDir.parentFile
val rustTargetDir = File(workspaceRoot, "target")
val bindingsDir = File(workspaceRoot, "crates/intl-ai-uniffi/bindings")

val hostOs = System.getProperty("os.name").lowercase()
val hostArch = System.getProperty("os.arch").lowercase()

fun normalizedArch(): String = when {
    hostArch in listOf("x86_64", "amd64") -> "x86-64"
    hostArch in listOf("aarch64", "arm64") -> "aarch64"
    hostArch in listOf("x86", "i386", "i686") -> "x86"
    hostArch.startsWith("arm") -> "arm"
    else -> hostArch
}

// JNA extracts libraries bundled in the JAR under com/sun/jna/<prefix>.
fun hostJnaPrefix(): String = when {
    hostOs.contains("linux") -> "linux-${normalizedArch()}"
    hostOs.contains("mac") || hostOs.contains("darwin") -> "darwin-${normalizedArch()}"
    hostOs.contains("windows") -> "win32-${normalizedArch()}"
    else -> error("unsupported host OS for bundling: $hostOs")
}

fun libFileName(): String = when {
    hostOs.contains("windows") -> "intl_ai_uniffi.dll"
    hostOs.contains("mac") || hostOs.contains("darwin") -> "libintl_ai_uniffi.dylib"
    else -> "libintl_ai_uniffi.so"
}

// Cross-build a different desktop target with e.g.
//   ./gradlew :intlai:build -PintlAiRustTarget=aarch64-apple-darwin -PintlAiJnaPrefix=darwin-aarch64
// (requires the Rust target and its linker to be installed).
val rustTarget = providers.gradleProperty("intlAiRustTarget").orNull
val jnaPrefix = providers.gradleProperty("intlAiJnaPrefix").getOrElse(hostJnaPrefix())
val rustReleaseDir =
    if (rustTarget != null) File(rustTargetDir, "$rustTarget/release") else File(rustTargetDir, "release")

val buildRustNative = tasks.register<Exec>("buildRustNative") {
    group = "build"
    description = "Compiles libintl_ai_uniffi (cdylib) for the host JVM target."
    workingDir = workspaceRoot
    commandLine(
        buildList {
            add("cargo")
            add("build")
            add("-p")
            add("intl-ai-uniffi")
            add("--release")
            rustTarget?.let { add("--target"); add(it) }
        }
    )
}

val bundleNativeLib = tasks.register<Copy>("bundleNativeLib") {
    group = "build"
    description = "Copies the native cdylib into the JAR at JNA's classpath location."
    dependsOn(buildRustNative)
    from(rustReleaseDir) { include(libFileName()) }
    // JNA resolves classpath-bundled natives at /<platform-prefix>/<libname>.
    into(layout.buildDirectory.dir("bundledNatives/$jnaPrefix"))
}

tasks.register<Exec>("generateKotlinBindings") {
    group = "build"
    description = "Regenerates the UniFFI Kotlin bindings into crates/intl-ai-uniffi/bindings."
    workingDir = workspaceRoot
    commandLine("bash", "crates/intl-ai-uniffi/scripts/generate-bindings.sh")
}

sourceSets["main"].apply {
    kotlin.srcDir(bindingsDir)
    resources.srcDir(layout.buildDirectory.dir("bundledNatives"))
}

tasks.named("processResources") { dependsOn(bundleNativeLib) }

kotlin {
    jvmToolchain(17)
}

dependencies {
    implementation("net.java.dev.jna:jna:5.18.1")
    testImplementation(kotlin("test"))
}

tasks.test {
    useJUnitPlatform()
}

publishing {
    publications {
        create<MavenPublication>("maven") {
            from(components["java"])
            pom {
                name.set("intlai")
                description.set("JVM bindings for the intl-ai i18n engine (UniFFI)")
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
