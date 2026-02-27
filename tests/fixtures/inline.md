# Inline Formatting Tests

## Basic Formatting

This has **bold text** in the middle.

This has *italic text* in the middle.

This has ~~strikethrough text~~ in the middle.

This has `inline code` in the middle.

## Combined Formatting

This is ***bold and italic*** together.

This is **bold with `code` inside**.

This is *italic with **bold** inside*.

This is ~~strikethrough with **bold** inside~~.

## Links and Images

Here is a [simple link](https://example.com) in text.

Here is a [link with **bold** text](https://example.com/bold).

![An example image](https://example.com/image.png)

An auto-link: https://www.example.com/auto

## Edge Cases

Empty bold: ****

Single character bold: **x**

Code with special chars: `<div class="foo">&amp;</div>`

Backticks in code: `` `hello` ``

Multiple inline codes: `one` and `two` and `three`.

A paragraph with no formatting at all, just plain text that should wrap nicely at the terminal width boundary without any issues whatsoever, demonstrating that basic text rendering works correctly.
