plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.android.library) apply false
    alias(libs.plugins.kotlin.android) apply false
    alias(libs.plugins.kotlin.compose) apply false
    alias(libs.plugins.kotlin.serialization) apply false
}

// `todo:ui` is how another build (the omnicui android target, through includeBuild) asks for the
// screens library.
subprojects { group = "todo" }
