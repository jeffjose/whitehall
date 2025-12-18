# Optimized Derived State with derivedStateOf

Tests derivedStateOf for performance-optimized computed state that only recomputes when dependencies change.

## Input

```whitehall
import $models.Product

  @prop val products: List<Product>

  var searchQuery = ""
  var minPrice = 0
  var maxPrice = 1000

  val filteredProducts: List<Product> = derivedStateOf {
    products.filter { product ->
      product.name.contains(searchQuery, ignoreCase = true) &&
      product.price >= minPrice &&
      product.price <= maxPrice
    }
  }

  val totalPrice: Double = derivedStateOf {
    filteredProducts.sumOf { it.price }
  }

<Column gap={16}>
  <TextField
    label="Search"
    bind:value={searchQuery}
    placeholder="Search products..."
  />

  <Row gap={8}>
    <TextField
      label="Min Price"
      bind:value={minPrice}
      type="number"
    />
    <TextField
      label="Max Price"
      bind:value={maxPrice}
      type="number"
    />
  </Row>

  <Text fontSize={16} fontWeight="bold">
    {filteredProducts.size} products - Total: ${totalPrice}
  </Text>

  @for (product in filteredProducts, key = { it.id }) {
    <Card>
      <Column padding={12}>
        <Text>{product.name}</Text>
        <Text color="secondary">${product.price}</Text>
      </Column>
    </Card>
  }
</Column>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.app.models.Product

@Composable
fun ProductFilter(
    products: List<Product>
) {
    var searchQuery by remember { mutableStateOf("") }
    var minPrice by remember { mutableStateOf(0) }
    var maxPrice by remember { mutableStateOf(1000) }

    val filteredProducts by remember {
        derivedStateOf {
            products.filter { product ->
            product.name.contains(searchQuery, ignoreCase = true) &&
            product.price >= minPrice &&
            product.price <= maxPrice
            }
            }
    }
    val totalPrice by remember {
        derivedStateOf {
            filteredProducts.sumOf { it.price }
            }
    }

    Column(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        TextField(
            label = { Text("Search") },
            value = searchQuery,
            onValueChange = { searchQuery = it },
            placeholder = { Text("Search products...") }
        )
        Row(
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            TextField(
                label = { Text("Min Price") },
                value = minPrice.toString(),
                onValueChange = { minPrice = it.toIntOrNull() ?: 0 },
                type = "number"
            )
            TextField(
                label = { Text("Max Price") },
                value = maxPrice.toString(),
                onValueChange = { maxPrice = it.toIntOrNull() ?: 1000 },
                type = "number"
            )
        }
        Text(
                text = "${filteredProducts.size} products - Total: \$${totalPrice}",
                fontSize = 16.sp,
                fontWeight = FontWeight.Bold
            )
        filteredProducts.forEach { product ->
            key(product.id) {
                Card {
                        Column(modifier = Modifier.padding(12.dp)) {
                            Text(text = "${product.name}")
                            Text(
                                text = "\$${product.price}",
                                color = MaterialTheme.colorScheme.secondary
                            )
                        }
                    }
            }
        }
    }
}
```

## Metadata

```
file: ProductFilter.wh
package: com.example.app.components
```
