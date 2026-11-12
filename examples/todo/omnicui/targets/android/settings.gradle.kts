// omnicui's android target: runs the todo app's Compose screens under Robolectric and answers
// omnicui's requests. The app is an included build, so this target always tests the app's
// current source; nothing is published.
pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}
dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
    // The app's versions, so that the target runs the same Compose as the app.
    versionCatalogs { create("libs") { from(files("../../../apps/android/gradle/libs.versions.toml")) } }
}
rootProject.name = "omnicui-target-android"
includeBuild("../../../apps/android")
