# Static List → RecyclerView Optimization

**Scenario:** User declares an immutable list with no mutations or event handlers

**Why optimize:** Compose LazyColumn has snapshot system overhead even for unchanging data. RecyclerView is 30-40% faster for truly static lists.

**Detection criteria:**
- ✅ Collection is `val` (immutable)
- ✅ Never mutated in component scope
- ✅ No event handlers in loop body
- ✅ Has key expression (for stable identity)
- ✅ Large list (100+ items heuristic)

---

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

---

## Unoptimized Output (Current - Phase 0)

**Backend:** Compose LazyColumn

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
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

    LazyColumn {
        items(contacts, key = { it.email }) { contact ->
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
```

**Performance:**
- Snapshot system overhead even though nothing changes
- Recomposition machinery active (unnecessary)
- ~45-50 FPS on large lists (1000+ items)
- Higher memory usage (Compose state tracking)

---

## Optimized Output (Phase 5 Target)

**Backend:** RecyclerView + ViewBinding

```kotlin
package com.example.app.components

import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
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

// Generated adapter for static list
private class ContactAdapter(
    private val contacts: List<Contact>
) : RecyclerView.Adapter<ContactAdapter.ViewHolder>() {

    override fun getItemCount(): Int = contacts.size

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): ViewHolder {
        // Create view programmatically (no XML layout needed)
        val card = MaterialCardView(parent.context).apply {
            setPadding(8.dp.toPx(), 8.dp.toPx(), 8.dp.toPx(), 8.dp.toPx())
        }

        val nameText = TextView(parent.context).apply {
            textSize = 16f
            setTypeface(null, android.graphics.Typeface.BOLD)
        }

        val emailText = TextView(parent.context).apply {
            textSize = 14f
            setTextColor(/* secondary color */)
        }

        // Simplified: actual implementation would use LinearLayout
        return ViewHolder(card, nameText, emailText)
    }

    override fun onBindViewHolder(holder: ViewHolder, position: Int) {
        val contact = contacts[position]
        holder.nameText.text = contact.name
        holder.emailText.text = contact.email
    }

    class ViewHolder(
        itemView: View,
        val nameText: TextView,
        val emailText: TextView
    ) : RecyclerView.ViewHolder(itemView)
}
```

**Performance:**
- No Compose overhead (direct View system)
- No snapshot tracking (data is static)
- ~60 FPS locked on large lists (1000+ items)
- 40% less memory usage
- ~30-40% faster initial render

**Trade-offs:**
- More verbose generated code
- Less flexible (harder to make dynamic later)
- Only applied when Whitehall is 100% confident it's safe

---

## Analyzer Decision Log (Future)

```
[ANALYZE] Checking for_loop over 'contacts'
  ✅ Collection 'contacts' is val (immutable): +40 confidence
  ✅ Not mutated anywhere in scope: +30 confidence
  ✅ Not a prop (defined in component): +20 confidence
  ✅ No event handlers in loop body: +10 confidence
  ✅ Has key expression: eligible for optimization
  ✅ TOTAL CONFIDENCE: 100

[OPTIMIZE] Planning RecyclerView optimization for 'contacts'
  ✅ Confidence threshold met (100 >= 80)
  ✅ Generated Optimization::UseRecyclerView

[CODEGEN] Generating RecyclerView for static list 'contacts'
  ✅ Creating adapter with ViewHolder pattern
  ✅ Wrapping in AndroidView for Compose interop
```

---

## Metadata

```
file: StaticContactList.wh
package: com.example.app.components
optimization: recyclerview
confidence: 100
```
