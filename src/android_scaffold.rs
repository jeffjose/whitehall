use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::config::Config;

/// Generate complete Android project scaffold
pub fn generate(config: &Config, output_dir: &Path) -> Result<()> {
    generate_root_gradle(output_dir)?;
    generate_settings_gradle(config, output_dir)?;
    generate_gradle_properties(output_dir)?;
    generate_app_gradle(config, output_dir)?;
    generate_manifest(config, output_dir)?;
    generate_proguard_rules(output_dir)?;

    Ok(())
}

/// Generate root build.gradle.kts
fn generate_root_gradle(output_dir: &Path) -> Result<()> {
    let content = r#"// Top-level build file where you can add configuration options common to all sub-projects/modules.
plugins {
    id("com.android.application") version "8.2.0" apply false
    id("org.jetbrains.kotlin.android") version "1.9.20" apply false
}
"#;
    fs::write(output_dir.join("build.gradle.kts"), content)?;
    Ok(())
}

/// Generate settings.gradle.kts
fn generate_settings_gradle(config: &Config, output_dir: &Path) -> Result<()> {
    let content = format!(
        r#"pluginManagement {{
    repositories {{
        google()
        mavenCentral()
        gradlePluginPortal()
    }}
}}

dependencyResolutionManagement {{
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {{
        google()
        mavenCentral()
    }}
}}

rootProject.name = "{}"
include(":app")
"#,
        config.project.name
    );

    fs::write(output_dir.join("settings.gradle.kts"), content)?;
    Ok(())
}

/// Generate gradle.properties
fn generate_gradle_properties(output_dir: &Path) -> Result<()> {
    let content = r#"# Project-wide Gradle settings.
org.gradle.jvmargs=-Xmx2048m -Dfile.encoding=UTF-8
org.gradle.parallel=true
org.gradle.caching=true

# Kotlin
kotlin.code.style=official

# AndroidX
android.useAndroidX=true
android.enableJetifier=false
"#;
    fs::write(output_dir.join("gradle.properties"), content)?;
    Ok(())
}

/// Generate app/build.gradle.kts
fn generate_app_gradle(config: &Config, output_dir: &Path) -> Result<()> {
    let content = format!(
        r#"plugins {{
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}}

android {{
    namespace = "{}"
    compileSdk = {}

    defaultConfig {{
        applicationId = "{}"
        minSdk = {}
        targetSdk = {}
        versionCode = 1
        versionName = "{}"

        vectorDrawables {{
            useSupportLibrary = true
        }}
    }}

    buildTypes {{
        release {{
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }}
    }}

    compileOptions {{
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }}

    kotlinOptions {{
        jvmTarget = "1.8"
    }}

    buildFeatures {{
        compose = true
    }}

    composeOptions {{
        kotlinCompilerExtensionVersion = "1.5.4"
    }}

    packaging {{
        resources {{
            excludes += "/META-INF/{{AL2.0,LGPL2.1}}"
        }}
    }}
}}

dependencies {{
    implementation("androidx.core:core-ktx:1.12.0")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.7.0")
    implementation("androidx.activity:activity-compose:1.8.2")

    implementation(platform("androidx.compose:compose-bom:2024.01.00"))
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-graphics")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("androidx.compose.material3:material3")

    // Navigation
    implementation("androidx.navigation:navigation-compose:2.7.6")

    // Coil for AsyncImage
    implementation("io.coil-kt:coil-compose:2.5.0")
}}
"#,
        config.android.package,
        config.android.target_sdk,
        config.android.package,
        config.android.min_sdk,
        config.android.target_sdk,
        config.project.version
    );

    fs::create_dir_all(output_dir.join("app"))?;
    fs::write(output_dir.join("app/build.gradle.kts"), content)?;
    Ok(())
}

/// Generate AndroidManifest.xml
fn generate_manifest(config: &Config, output_dir: &Path) -> Result<()> {
    let content = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android">

    <uses-permission android:name="android.permission.INTERNET" />

    <application
        android:allowBackup="true"
        android:label="{}"
        android:supportsRtl="true"
        android:theme="@android:style/Theme.Material.Light.NoActionBar">
        <activity
            android:name=".MainActivity"
            android:exported="true"
            android:theme="@android:style/Theme.Material.Light.NoActionBar">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>
    </application>

</manifest>
"#,
        config.project.name
    );

    let manifest_dir = output_dir.join("app/src/main");
    fs::create_dir_all(&manifest_dir)?;
    fs::write(manifest_dir.join("AndroidManifest.xml"), content)?;
    Ok(())
}

/// Generate proguard-rules.pro
fn generate_proguard_rules(output_dir: &Path) -> Result<()> {
    let content = r#"# Add project specific ProGuard rules here.
# You can control the set of applied configuration files using the
# proguardFiles setting in build.gradle.
#
# For more details, see
#   http://developer.android.com/guide/developing/tools/proguard.html

# Keep Compose classes
-keep class androidx.compose.** { *; }
-dontwarn androidx.compose.**

# Keep Kotlin metadata
-keep class kotlin.Metadata { *; }

# Keep serialization classes (for Navigation)
-keepattributes *Annotation*, InnerClasses
-dontnote kotlinx.serialization.AnnotationsKt
-keepclassmembers class kotlinx.serialization.json.** {
    *** Companion;
}
-keepclasseswithmembers class kotlinx.serialization.json.** {
    kotlinx.serialization.KSerializer serializer(...);
}
"#;

    fs::write(output_dir.join("app/proguard-rules.pro"), content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AndroidConfig, BuildConfig, Config, FfiConfig, ProjectConfig, ToolchainConfig};
    use tempfile::TempDir;

    fn make_test_config() -> Config {
        Config {
            project: ProjectConfig {
                name: "TestApp".to_string(),
                version: "1.0.0".to_string(),
            },
            android: AndroidConfig {
                min_sdk: 24,
                target_sdk: 34,
                package: "com.example.testapp".to_string(),
            },
            build: BuildConfig::default(),
            toolchain: ToolchainConfig::default(),
            ffi: FfiConfig::default(),
        }
    }

    #[test]
    fn test_generate_scaffold() {
        let temp = TempDir::new().unwrap();
        let config = make_test_config();

        let result = generate(&config, temp.path());
        assert!(result.is_ok());

        // Verify key files exist
        assert!(temp.path().join("build.gradle.kts").exists());
        assert!(temp.path().join("settings.gradle.kts").exists());
        assert!(temp.path().join("gradle.properties").exists());
        assert!(temp.path().join("app/build.gradle.kts").exists());
        assert!(temp.path().join("app/src/main/AndroidManifest.xml").exists());
        assert!(temp.path().join("app/proguard-rules.pro").exists());
    }

    #[test]
    fn test_settings_gradle_contains_project_name() {
        let temp = TempDir::new().unwrap();
        let config = make_test_config();

        generate_settings_gradle(&config, temp.path()).unwrap();

        let content = fs::read_to_string(temp.path().join("settings.gradle.kts")).unwrap();
        assert!(content.contains("TestApp"));
    }

    #[test]
    fn test_manifest_contains_package() {
        let temp = TempDir::new().unwrap();
        let config = make_test_config();

        generate_manifest(&config, temp.path()).unwrap();

        let content =
            fs::read_to_string(temp.path().join("app/src/main/AndroidManifest.xml")).unwrap();
        assert!(content.contains("TestApp"));
    }

    #[test]
    fn test_app_gradle_contains_sdk_versions() {
        let temp = TempDir::new().unwrap();
        let config = make_test_config();

        generate_app_gradle(&config, temp.path()).unwrap();

        let content = fs::read_to_string(temp.path().join("app/build.gradle.kts")).unwrap();
        assert!(content.contains("minSdk = 24"));
        assert!(content.contains("targetSdk = 34"));
        assert!(content.contains("com.example.testapp"));
    }
}
