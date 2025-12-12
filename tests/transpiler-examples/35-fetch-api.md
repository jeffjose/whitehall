# Fetch API for HTTP Requests

Tests the fetch() API which transforms to Ktor HttpClient calls.

## Input

```whitehall
fun loadData(): String {
  return fetch("https://api.example.com/data")
}

<Text>Click to load</Text>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.engine.okhttp.OkHttp
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.get
import io.ktor.serialization.kotlinx.json.json
import kotlinx.serialization.json.Json

private val httpClient = HttpClient(OkHttp) {
    install(ContentNegotiation) {
        json(Json { ignoreUnknownKeys = true })
    }
}

@Composable
fun FetchTest() {
    fun loadData(): String {
        return httpClient.get("https://api.example.com/data").body()
    }

    Text(text = "Click to load")
}
```

## Metadata

```
file: FetchTest.wh
package: com.example.app.components
```

## Notes

The fetch() API provides a web-like syntax for HTTP requests:
- `fetch(url)` transforms to `httpClient.get(url).body()`
- HttpClient singleton is generated at file level
- Uses Ktor with OkHttp engine for Android
- Kotlinx.serialization for JSON parsing
- Type inference from variable annotation (e.g., `List<Photo>`)
