// The todo app for Android: `ui` holds the screens (a library, which the omnicui android target
// also depends on), `app` the activity that shows them.
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
}
rootProject.name = "todo-android"
include(":ui", ":app")
