# Static List Optimization

Tests RecyclerView optimization for static immutable lists.

## Input

```whitehall
val fruits = listOf("Apple", "Banana", "Cherry")

<Column>
  @for (fruit in fruits) {
    <Text text={fruit} />
  }
</Column>
```

## Unoptimized Output

```kotlin
package com.example

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text
import androidx.compose.runtime.*

@Composable
fun StaticList() {
    val fruits = listOf("Apple", "Banana", "Cherry")

    Column {
        fruits.forEach { fruit ->
            Text(text = fruit)
        }
    }
}
```

## Optimized Output

```kotlin
package com.example

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text
import androidx.compose.runtime.*

@Composable
fun StaticList() {
    val fruits = listOf("Apple", "Banana", "Cherry")

    Column {
        AndroidView(
            factory = { context ->
                RecyclerView(context).apply {
                    layoutManager = LinearLayoutManager(context)
                    adapter = object : RecyclerView.Adapter<RecyclerView.ViewHolder>() {
                        override fun getItemCount() = fruits.size

                        override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): RecyclerView.ViewHolder {
                            val view = TextView(parent.context).apply {
                                layoutParams = ViewGroup.LayoutParams(
                                    ViewGroup.LayoutParams.MATCH_PARENT,
                                    ViewGroup.LayoutParams.WRAP_CONTENT
                                )
                                setPadding(16.dpToPx(), 16.dpToPx(), 16.dpToPx(), 16.dpToPx())
                            }
                            return object : RecyclerView.ViewHolder(view) {}
                        }

                        override fun onBindViewHolder(holder: RecyclerView.ViewHolder, position: Int) {
                            val fruit = fruits[position]
                            val textView = holder.itemView as TextView
                            textView.text = fruit.toString()
                        }
                    }
                }
            }
        )

        // Extension for DP to PX conversion
        private fun Int.dpToPx(): Int {
            val density = Resources.getSystem().displayMetrics.density
            return (this * density).toInt()
        }
    }
}
```

## Metadata

```
file: StaticList.wh
package: com.example
```
