plugins {
    id("com.android.library") version "8.7.0"
    kotlin("android") version "2.0.20"
}
android {
    namespace = "io.github.aghorbani84.aurex.storage"
    compileSdk = 35
    defaultConfig { minSdk = 24 }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions { jvmTarget = "17" }
}
dependencies {
    implementation("androidx.activity:activity-ktx:1.9.3")
}
