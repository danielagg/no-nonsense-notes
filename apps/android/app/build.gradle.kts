plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "app.nononsense.notes"
    compileSdk = 34

    defaultConfig {
        applicationId = "app.nononsense.notes"
        minSdk = 26
        targetSdk = 34
        versionCode = 1
        versionName = "0.1.0"
    }

    val debugApiUrl = providers.gradleProperty("debugApiUrl")
        .orElse("http://10.0.2.2:3000")
    val releaseApiUrl = providers.gradleProperty("releaseApiUrl")
        .orElse("https://no-nonsense-notes.onrender.com")
    val releaseKeystore = providers.environmentVariable("NNN_ANDROID_KEYSTORE").orNull

    signingConfigs {
        if (releaseKeystore != null) {
            create("release") {
                storeFile = file(releaseKeystore)
                storePassword = providers.environmentVariable("NNN_ANDROID_KEYSTORE_PASSWORD").orNull
                keyAlias = providers.environmentVariable("NNN_ANDROID_KEY_ALIAS").orNull
                keyPassword = providers.environmentVariable("NNN_ANDROID_KEY_PASSWORD").orNull
            }
        }
    }

    buildTypes {
        debug {
            buildConfigField("String", "API_URL", "\"${debugApiUrl.get()}\"")
            manifestPlaceholders["usesCleartextTraffic"] = "true"
        }
        release {
            buildConfigField("String", "API_URL", "\"${releaseApiUrl.get()}\"")
            manifestPlaceholders["usesCleartextTraffic"] = "false"
            signingConfig = signingConfigs.findByName("release")
            isMinifyEnabled = false
        }
    }

    buildFeatures { compose = true; buildConfig = true }
    composeOptions { kotlinCompilerExtensionVersion = "1.5.8" }
    packaging { resources.excludes += "/META-INF/{AL2.0,LGPL2.1}" }
    kotlinOptions { jvmTarget = "17" }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

val buildRust by tasks.registering(Exec::class) {
    workingDir = rootProject.projectDir.parentFile.parentFile
    commandLine("./scripts/build-android-rust.sh")
}
tasks.named("preBuild").configure { dependsOn(buildRust) }

dependencies {
    implementation("androidx.core:core-ktx:1.13.1")
    implementation("androidx.core:core-splashscreen:1.0.1")
    implementation("androidx.activity:activity-compose:1.9.0")
    implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.8.0")
    implementation("androidx.compose.ui:ui:1.6.8")
    implementation("androidx.compose.ui:ui-tooling-preview:1.6.8")
    implementation("androidx.compose.foundation:foundation:1.6.8")
    implementation("androidx.compose.material3:material3:1.2.1")
    implementation("androidx.compose.material:material-icons-extended:1.6.8")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.8.1")
    implementation("net.java.dev.jna:jna:5.14.0@aar")
    debugImplementation("androidx.compose.ui:ui-tooling:1.6.8")
}
