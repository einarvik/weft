# Weft architecture

## Compilation model

```text
.wft source
  -> lexer / indentation parser
  -> validated semantic AST
  -> HTML renderer + CSS renderer + optional island renderer
  -> dist/index.html, dist/site.css, optional dist/islands.js
```

The AST is the compiler’s source of truth. Generated files are reproducible outputs, not hand-maintained source.

## Rendering policy

- Pages and ordinary components render as semantic HTML.
- Layout and visual intent render as raw CSS.
- A document without islands contains no script tag.
- An island has a local, explicit client boundary.
- Generated markup must be accessible by default: landmarks, valid heading order, keyboard-visible focus, and controls with names.

## Why Rust

Weft is implemented in Rust for a fast compiler, a self-contained distribution path, robust parsing/error handling, and a strong ecosystem for future static analysis and code generation. The runtime output remains browser-native; Rust is a compiler implementation choice, not a browser runtime dependency.

## Future integration boundary

Weft initially emits web standards directly. Adapters for runtimes or deployment systems can later be added behind the renderer boundary, without changing the source language or making any framework part of the generated page contract.
