# List Tests

## Simple Unordered

- Apple
- Banana
- Cherry

## Simple Ordered

1. First
2. Second
3. Third

## Nested Unordered (3 levels)

- Level 0 item A
  - Level 1 item A
    - Level 2 item A
    - Level 2 item B
  - Level 1 item B
- Level 0 item B
  - Level 1 item C

## Nested Ordered

1. First outer
   1. First inner
   2. Second inner
2. Second outer
   1. Another inner

## Mixed Lists

1. Ordered item
   - Unordered child
   - Another unordered child
2. Another ordered
   - Child with **bold**
     1. Deep ordered
     2. Deep ordered two

## Task Lists

- [x] Buy groceries
- [ ] Clean the house
- [x] Write documentation
- [ ] Review pull requests

## Tight vs Loose Lists

Tight list:

- One
- Two
- Three

Loose list:

- One

- Two

- Three

## Long List Items

- This is a very long list item that should demonstrate how the renderer handles list items that contain enough text to potentially wrap across multiple lines in the terminal output.
- Short item
- Another long item with **bold formatting** and `inline code` mixed in to test that styled content within list items renders properly.
