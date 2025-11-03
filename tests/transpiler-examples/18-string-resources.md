# String Resources for i18n

Tests string resource references for internationalization support.

## Input

```whitehall
  @prop val userName: String
  @prop val itemCount: Int

<Column spacing={16}>
  <Text fontSize={24} fontWeight="bold">
    {R.string.welcome_title}
  </Text>

  <Text color="secondary">
    {R.string.greeting(userName)}
  </Text>

  <Text>
    {R.string.items_count(itemCount)}
  </Text>

  <Button
    text={R.string.action_continue}
    onClick={() => navigate($routes.home)}
  />
</Column>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.app.R

@Composable
fun WelcomeScreen(
    userName: String,
    itemCount: Int
) {
    Column(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Text(
            text = stringResource(R.string.welcome_title),
            fontSize = 24.sp,
            fontWeight = FontWeight.Bold
        )

        Text(
            text = stringResource(R.string.greeting, userName),
            color = MaterialTheme.colorScheme.secondary
        )

        Text(
            text = stringResource(R.string.items_count, itemCount)
        )

        Button(
            onClick = { navigate(Routes.Home) }
        ) {
            Text(text = stringResource(R.string.action_continue))
        }
    }
}
```

## Metadata

```
file: WelcomeScreen.wh
package: com.example.app.components
```
