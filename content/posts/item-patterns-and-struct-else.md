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

# Items and sadness

Items have a hard life. They are the parents of everything important. `struct`, `enum`, `const`, `mod`, `fn`, `union`, `global_asm` are all things we use daily, yet their grammar is very limited.


For example, see the following code where we declare a few constants.

```
const ONE: u8 = 1;
const TWO: u8 = 1;
const THREE: u8 = 3;
```

There is nothing obviously wrong with this code. You understand it, I understand it, an ALGOL 68 developer from 1970 would probably understand it
and even an ancient greek philopher might have a clue (which is impressive, given that they are all not alive anymore). But this is the kind of code that pages you at 4 AM.

You've read the last paragraph in confusion. Of course there's something wrong with this code! `TWO` is `1`, yet the name strongly suggests that it should be `2`. And you'd
be right, this was just a check to make sure you're still here.

But even if it was `2`, this code is still not good. There is way too much duplication! `const` is mentioned three times. This is a major distraction to the reader.

Let's have a harder example:

```
const ONE: u8 = 0; const
NAME: &
str = "nils";
       const X: &str
  =   "const";const A: () = ();
```

Here, the `const` being noise is a lot more obvious. Did you see that `X` contains `"const"`? Maybe you did, maybe you didn't. When I tested it, 0/0 people could see it.

Now imagine if it looked like this:

```rust
const (ONE, NAME, X, A): (u8, &str, &str, ()) = (0, "nils", "const", ());
```

Everything is way shorter and more readable.

What you've just seen is a limited form of pattern matching!

# Let's go further

The idea of generalizing pattern matching is very powerful. We can apply this to more than just consts.

```rust
struct (Person, Car) = ({ name: String }, { wheels: u8 });
```

Here, we create two structs with just a single `struct` keyword. This makes it way simpler and easier to read when related structs are declared.
So far we've just used tuples. But we can go even further. Metastructs!

```rust
struct Household<T, U> {
  parent: T,
  child: U,
}

struct Household { parent: Ferris, child: Corro } = Household {
  parent: { name: String },
  child: { name: String, unsafety: bool },
};
```

Now we can nicely patch on the `Household` metastruct containing the definition of the `Ferris` and `Corro` structs. This is equivalent to the following code:

```rust
struct Feris {
  name: String,
}

struct Corro {
  name: String,
  unsafety: bool,
}
```

This is already really neat, but there's more. We also have to consider the falliblity of patterns.

```rust
static Some(A) = None;
```

This pattern doesn't match. Inside bodies, we could use an `if let`:

```rust
if let Some(a) = None {} else {}
```

We can also apply this to items.

```rust
if struct Some(A) = None {
  /* other items where A exists */
} else {
  /* other items where A doesn't exist */
}
```

This doesn't sound too useful, it allows for extreme flexibility!

```rust
macro_rules! are_same_type {
  ($a:ty, $b:ty) => {{
    static mut ARE_SAME: bool = false;
  
    if struct $a = $b {
      const _: () = unsafe { ARE_SAME = true; };
    }
    
    unsafe { ARE_SAME }
  }};
}

fn main() {
  if are_same_type!(Vec<String>, String) {
    println!("impossible to reach!");
  }
}
```

We can go further.

Today, items are just there with no ordering. What if we imposed an ordering? What if "rust items" was a meta scripting language?

We can write a simple guessing game!

```rust
struct fn input() -> u8 {
  const INPUT: &str = prompt!();
  const Ok(INPUT): Result<u8, ParseIntErr> = INPUT.parse() else {
    compile_error!("Invalid input");
  };
  INPUT
}

const RANDOM: u8 = env!("RANDOM");

loop {
  const INPUT = input();
  if INPUT == RANDOM {
    break; // continue compilation
  } else if INPUT < RANDOM {
    compile_warn!("input is smaller");
  } else {
    compile_warn!("input is bigger");
  }
}

fn main() {
  // Empty. I am useless.
}
```

And then, last but not least I want to highlight one of my favourite consequences of this: `struct else`

```rust
struct Some(Test) = None else {
  compile_error!("didn't match pattern");
};

<sub>this post was not meant to make fun of anyone's ideas. it was just a good idea i had once and then friends made me write this</sub>
