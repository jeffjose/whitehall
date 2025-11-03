# Function Return Type Annotations

## Input (.wh)

```whitehall
fun getMessage(): String {
  return "Hello, World!"
}

fun getCount(x: Int): Int {
  return x + 1
}

<Column>
  <Text>{getMessage()}</Text>
  <Text>{getCount(5)}</Text>
</Column>
```

## Expected Output (.kt)

```kotlin
@Composable
fun Screen() {
    fun getMessage(): String {
        return "Hello, World!"
    }

    fun getCount(x: Int): Int {
        return x + 1
    }

    Column {
        Text(getMessage())
        Text(getCount(5).toString())
    }
}
```
