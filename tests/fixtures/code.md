# Code Block Tests

## Rust

```rust
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert("key", 42);

    match map.get("key") {
        Some(val) => println!("Found: {}", val),
        None => println!("Not found"),
    }

    let numbers: Vec<i32> = (0..10).filter(|x| x % 2 == 0).collect();
    println!("{:?}", numbers);
}
```

## JavaScript

```javascript
async function fetchData(url) {
  try {
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    const data = await response.json();
    return data;
  } catch (error) {
    console.error('Fetch failed:', error);
  }
}

const result = await fetchData('https://api.example.com/data');
```

## Python

```python
from dataclasses import dataclass
from typing import Optional

@dataclass
class User:
    name: str
    email: str
    age: Optional[int] = None

    def greet(self) -> str:
        return f"Hello, {self.name}!"

users = [User("Alice", "alice@example.com", 30), User("Bob", "bob@example.com")]
for user in users:
    print(user.greet())
```

## Shell

```bash
#!/bin/bash
set -euo pipefail

for file in *.md; do
    echo "Processing: $file"
    wc -l "$file"
done | sort -rn | head -5
```

## No Language Specified

```
This is a plain code block
with no language specified.
It should render without syntax highlighting.
```

## Short Code Block

```go
fmt.Println("Hello, World!")
```

## Empty Code Block

```

```

## Code Block with Very Long Lines

```
This is a line that is intentionally very long to test how the code block renderer handles content that exceeds the terminal width - it should probably clip or show as-is within the box.
Short line after.
```
