
---
id: whitehall-971
title: "$fetch not transformed in regular functions (only works in onMount)"
type: bug
status: closed
priority: high
created: 2025-01-13
parent: whitehall-970
---

## Description

The `$fetch` syntax is only transformed to proper Ktor HTTP client code when used inside `onMount` blocks. When used in regular functions, it's passed through unchanged, causing Kotlin compilation errors.

## Current Behavior

```whitehall
fun loadMore() {
  val newPhotos = $fetch("https://picsum.photos/v2/list")
}
```

Generates invalid Kotlin:
```kotlin
fun loadMore() {
  val newPhotos = $fetch("https://picsum.photos/v2/list")  // $fetch not valid Kotlin
}
```

## Expected Behavior

Should generate proper Ktor client call:
```kotlin
fun loadMore() {
  viewModelScope.launch {
    val newPhotos: List<Photo> = httpClient.get("https://picsum.photos/v2/list").body()
  }
}
```

## Impact

Blocks infinite scroll / pagination features that require fetching in user-defined functions.
