import org.jetbrains.compose.desktop.application.dsl.TargetFormat
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.androidApplication)
    alias(libs.plugins.composeMultiplatform)
    alias(libs.plugins.composeCompiler)
    alias(libs.plugins.composeHotReload)
}

kotlin {
    androidTarget {
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_11)
        }
    }
    
    jvm()
    
    sourceSets {
        androidMain.dependencies {
            implementation(libs.compose.uiToolingPreview)
            implementation(libs.androidx.activity.compose)

            implementation(libs.koin.android)

            implementation(libs.androidx.media3.session)
            implementation (libs.commons.math3)

            implementation(libs.androidx.documentfile)

            // JNA, которой uniffi грузит libplayer_core: на Android нужен именно aar —
            // в нём лежат нативные части JNA под все ABI.
            implementation("net.java.dev.jna:jna:5.14.0@aar")
        }
        commonMain.dependencies {
            implementation(libs.compose.runtime)
            implementation(libs.compose.foundation)
            implementation(libs.compose.material3)
            implementation(libs.compose.ui)

            implementation(libs.compose.components.resources)
            implementation(libs.compose.uiToolingPreview)
            implementation(libs.androidx.lifecycle.viewmodelCompose)
            implementation(libs.androidx.lifecycle.runtimeCompose)

            implementation(libs.koin.core)
            implementation(libs.koin.compose)
            implementation(libs.koin.compose.viewmodel)

            implementation(libs.androidx.datastore.preferences)
        }
        commonTest.dependencies {
            implementation(libs.kotlin.test)
        }
        jvmMain.dependencies {
            implementation(compose.desktop.currentOs)
            implementation(libs.kotlinx.coroutinesSwing)
            implementation("org.jetbrains.skiko:skiko-awt-runtime-windows-x64:0.9.3")
            // Та же JNA, но обычным jar: aar — упаковка для Android, JVM её не читает.
            implementation("net.java.dev.jna:jna:5.14.0")
            implementation(compose.desktop.windows_x64)
        }
    }
}

android {
    namespace = "wifi.music.list"
    compileSdk = libs.versions.android.compileSdk.get().toInt()

    defaultConfig {
        applicationId = "wifi.music.list"
        minSdk = libs.versions.android.minSdk.get().toInt()
        targetSdk = libs.versions.android.targetSdk.get().toInt()
        versionCode = 1
        versionName = "1.0"
    }
    packaging {
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
    }
    buildTypes {
        getByName("release") {
            isMinifyEnabled = false
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    lint {
        abortOnError = false
        checkReleaseBuilds = false
    }
}

dependencies {
    debugImplementation(libs.compose.uiTooling)
}

compose.desktop {
    application {
        mainClass = "wifi.music.list.MainKt"

        nativeDistributions {
            targetFormats(TargetFormat.Dmg, TargetFormat.Msi, TargetFormat.Deb, TargetFormat.AppImage)
            packageName = "wifi.music.list"
            packageVersion = "1.0.0"
            modules("jdk.unsupported")
            appResourcesRootDir.set(project.layout.projectDirectory.dir("desktop-resources"))

            windows {
                iconFile.set(project.file("src/jvmMain/resources/image.ico"))
                appResourcesRootDir.set(project.file("desktop-resources"))
                menu = true
            }
            linux {
                iconFile.set(project.file("src/jvmMain/resources/image.png"))
                appResourcesRootDir.set(project.file("desktop-resources"))
            }
        }
    }
    tasks.withType<JavaExec>().configureEach {
        // линукс
        systemProperty("jna.library.path", file("./src/main/rust/target/release").absolutePath)
        // винда
        // systemProperty("jna.library.path", file("./src/main/rust/target/x86_64-pc-windows-gnu").absolutePath)
    }

    tasks.register<Jar>("packageWindowsFatJar") {
        group = "package"
        description = "Assembles Fat JAR for Windows"

        archiveClassifier.set("windows")

        duplicatesStrategy = DuplicatesStrategy.EXCLUDE

        val jvmTarget = kotlin.targets.getByName("jvm")
        val mainCompilation = jvmTarget.compilations.getByName("main")

        from(mainCompilation.output.classesDirs)
        from(mainCompilation.output.resourcesDir)

        dependsOn(mainCompilation.runtimeDependencyFiles)
        from({
            mainCompilation.runtimeDependencyFiles
                ?.filter { it.name.endsWith(".jar") }
                ?.map { zipTree(it) }
        }) {
            exclude("META-INF/*.SF", "META-INF/*.DSA", "META-INF/*.RSA")
        }

        manifest {
            attributes["Main-Class"] = "wifi.music.list.MainKt"
        }
    }
}