# Okio & DataStore
-keep class okio.** { *; }
-dontwarn okio.**

-keep class androidx.datastore.** { *; }
-dontwarn androidx.datastore.**

# Kotlin Coroutines
-keepnames class kotlinx.coroutines.internal.MainDispatcherFactory {}
-keepnames class kotlinx.coroutines.CoroutineExceptionHandler {}
-keepclassmembers class kotlinx.coroutines.** { volatile <fields>; }

# JNA & Native
-keep class com.sun.jna.** { *; }
-keepclassmembers class com.sun.jna.** { *; }
-dontwarn com.sun.jna.**

-keepclasseswithmembernames,includedescriptorclasses class * {
    native <methods>;
}

# UniFFI
-keep class uniffi.** { *; }
-keepclassmembers class uniffi.** { *; }

# Koin
-keep class org.koin.** { *; }
-dontwarn org.koin.**