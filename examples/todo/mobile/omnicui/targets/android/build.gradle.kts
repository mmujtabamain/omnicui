import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.compose)
}

android {
    namespace = "omnicui.android"
    compileSdk = 36
    defaultConfig { minSdk = 26 }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    buildFeatures { compose = true }
    // Robolectric reads the merged manifest and resources of this variant (the app's included).
    testOptions { unitTests.isIncludeAndroidResources = true }
}

kotlin { compilerOptions { jvmTarget.set(JvmTarget.JVM_17) } }

dependencies {
    // The app under test: its screens library, from the included build ../../../apps/android.
    implementation("todo:ui")
    implementation(platform(libs.compose.bom))

    testImplementation(platform(libs.compose.bom))
    testImplementation("androidx.compose.ui:ui-test-junit4")
    testImplementation("org.robolectric:robolectric:4.16.1")
    testImplementation(libs.junit)
    // ComponentActivity in the test manifest, for runAndroidComposeUiTest.
    debugImplementation("androidx.compose.ui:ui-test-manifest")
}

// The adapter runs as a plain JVM, not through Gradle: `./gradlew adapterClasspath` (setup.sh)
// compiles it once and writes the exact classpath the unit tests run with into
// build/adapter.args, which target.yaml passes to `java @build/adapter.args`.
val adapterClasspath = tasks.register("adapterClasspath") {
    val test = tasks.named<Test>("testDebugUnitTest")
    val classpath = test.map { it.classpath }
    inputs.files(classpath)
    val out = layout.buildDirectory.file("adapter.args")
    outputs.file(out)
    doLast {
        val cp = classpath.get().files.joinToString(File.pathSeparator) { it.absolutePath }
        out.get().asFile.writeText("-cp \"$cp\"\n")
    }
}
