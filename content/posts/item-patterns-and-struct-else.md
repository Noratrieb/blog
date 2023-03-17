+++
title = "Item Patterns And Struct Else"
date = "2023-03-17"
author = "Nilstrieb"
authorTwitter = "@Nilstrieb"
tags = ["rust", "language-design"]
keywords = ["design"]
description = "Bringing more expressiveness to our items"
showFullContent = false
readingTime = true
hideComments = false
draft = true
+++

# Pattern matching

One of my favourite features of Rust is pattern matching. It's a simple and elegant way to deal with not just structs, but also enums!

```rust
enum ItemKind {
  Struct(String, Vec<Field>),
  Function(String, Body),
}

impl ItemKind {
  fn name(&self) -> &str {
    match self {
      Self::Struct(name, _) => name,
      Self::Function(name, _) => name,
    }
  }
}
```

Here, we have an enum and a function to get the name out of this. In C, this would be very unsafe, as we cannot be guaranteed that our union has the right tag.
But in Rust, the compiler nicely checks it all for us. It's safe and expressive (just like many other features of Rust).

But that isn't the only way to use pattern matching. While branching is one of its core features, it doesn't always have to be used. Another major advantage of pattern matching
lies in the ability to _exhaustively_ match over inputs.

Let's look at the following example. Here, we have a struct representing a struct in a programming language. It has a name and fields.
We then manually implement a custom hash trait for it. We could have written a derive macro, but didn't.

```rust 
struct Struct {
  name: String,
  fields: Vec<Field>,
}

impl HandRolledHash for Struct {
  fn hash(&self, hasher: &mut HandRolledHasher) {
    hasher.hash(&self.name);
    hasher.hash(&self.fields);
  }
}
```

This works perfectly. But then later, [we add privacy to the language](https://github.com/rust-lang/rustup/pull/1642). Now, all types have a visibility.

```diff
struct Struct {
+  visibility: Vis,
  name: String,
  fields: Vec<Field>,
}
```

Pretty cool. Now no one can access the implementation details and make everything a mess. But wait - we have just made a mess! We didn't hash the privacy!
Hashing something incorrectly [doesn't sound too bad](https://github.com/rust-lang/rust/issues/84970), but it would be nice if this was prevented.

Thanks to exhaustive pattern matching, it would have been easy to prevent. We just change our hash implementation:

```rust
impl HandRolledHash for Struct {
  fn hash(&self, hasher: &mut HandRolledHasher) {
    let Self { name, fields } = self;
    hasher.hash(name);
    hasher.hash(fields);
  }
}
```

And with this, adding the visibility will cause a compiler error and alert us that we need to handle it in hashing.

We can conclude that pattern matching is a great feature.

# Limitations of pattern matching

But there is one big limitation of pattern matching - all of its occurrences (`match`, `if let`, `if let` chains, `while let`, `for`, `let`, `let else`, function parameters
(we do have a lot of pattern matching)) are inside of bodies, mostly as part of expressions or statements.

This doesn't sound too bad. This is where the executed code resides. But it comes at a cost of consistency. We often add many syntactical niceties to expressions and statements, but forget about items.

Items have a hard life. They are the parents of everything important. `struct`, `enum`, `const`, `mod`, `fn`, `union`, `global_asm` are all things we use daily, yet their grammar is very limited.


For example, see the following code

```

<sub>maybe this post was meant as a joke. maybe it wasn't. it is up to you to bring your own judgement to the idea and write an RFC.</sub>
