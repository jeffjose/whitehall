# Static List Optimization (Future)

Tests RecyclerView optimization for static immutable lists.

**Unoptimized:** Generates Compose LazyColumn (Phase 0-4)
**Optimized:** Generates RecyclerView when confidence >= 80 (Phase 5+)

**Optimization criteria:**
- Collection is `val` (immutable)
- Never mutated in scope
- No event handlers
- Has key expression
- Confidence: 100/100

## Input

```whitehall
val contacts = listOf(
  Contact("Alice", "alice@example.com"),
  Contact("Bob", "bob@example.com"),
  Contact("Charlie", "charlie@example.com")
)

@for (contact in contacts, key = { it.email }) {
  <Card padding={8}>
    <Column spacing={4}>
      <Text fontSize={16} fontWeight="bold">{contact.name}</Text>
      <Text fontSize={14} color="secondary">{contact.email}</Text>
    </Column>
  </Card>
}
```

## Unoptimized Output

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun StaticContactList() {
    val contacts = listOf(
        Contact("Alice", "alice@example.com"),
        Contact("Bob", "bob@example.com"),
        Contact("Charlie", "charlie@example.com")
    )

    Column {
        contacts.forEach { contact ->
            key(contact.email) {
                Card(modifier = Modifier.padding(8.dp)) {
                    Column(
                        verticalArrangement = Arrangement.spacedBy(4.dp)
                    ) {
                        Text(
                            text = contact.name,
                            fontSize = 16.sp,
                            fontWeight = FontWeight.Bold
                        )
                        Text(
                            text = contact.email,
                            fontSize = 14.sp,
                            color = MaterialTheme.colorScheme.secondary
                        )
                    }
                }
            }
        }
    }
}
```

## Optimized Output

```kotlin
package com.example.app.components

import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.compose.runtime.Composable
import androidx.compose.ui.viewinterop.AndroidView
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.google.android.material.card.MaterialCardView

@Composable
fun StaticContactList() {
    val contacts = listOf(
        Contact("Alice", "alice@example.com"),
        Contact("Bob", "bob@example.com"),
        Contact("Charlie", "charlie@example.com")
    )

    AndroidView(
        factory = { context ->
            RecyclerView(context).apply {
                layoutManager = LinearLayoutManager(context)
                adapter = ContactAdapter(contacts)
            }
        }
    )
}

// Generated adapter for static list optimization
private class ContactAdapter(
    private val contacts: List<Contact>
) : RecyclerView.Adapter<ContactAdapter.ViewHolder>() {

    override fun getItemCount(): Int = contacts.size

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): ViewHolder {
        val card = MaterialCardView(parent.context)
        // View creation code...
        return ViewHolder(card)
    }

    override fun onBindViewHolder(holder: ViewHolder, position: Int) {
        val contact = contacts[position]
        holder.nameText.text = contact.name
        holder.emailText.text = contact.email
    }

    class ViewHolder(itemView: View) : RecyclerView.ViewHolder(itemView) {
        val nameText: TextView = itemView.findViewById(R.id.name)
        val emailText: TextView = itemView.findViewById(R.id.email)
    }
}
```

**Performance improvement:** 30-40% faster scroll, 40% less memory for large lists

## Metadata

```
file: StaticContactList.wh
package: com.example.app.components
optimization: recyclerview
confidence: 100
phase: 5
```
