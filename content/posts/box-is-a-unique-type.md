+++
title = "Box Is a Unique Type"
date = "2022-07-21T17:34:24+02:00"
author = "Nilstrieb"
authorTwitter = "@Nilstrieb"
cover = ""
tags = ["", ""]
keywords = ["", ""]
description = ""
showFullContent = false
readingTime = true
hideComments = false
draft = true
+++

We have all used `Box<T>` before in our Rust code. It's a glorious type, with great ergonomics
and flexibitility. We can use it to simply put our values on the heap, but it can do even more
than that!

```rust
struct Fields {
    a: String,
    b: String,
}

let fields = Box::new(Fields { 
    a: "a".to_string(), 
    b: "b".to_string() 
});

let a = fields.a;
let b = fields.b;
```

This kind of partial deref move is just one of the spectacular magic tricks box has up its sleeve,
and they exist for good reason: They are very useful. Sadly we have not yet found a way to generalize all
of these to user types as well. Too bad!

Anyways, this post is about one particularly subtle magic aspect of box. For this, we need to dive
deep into unsafe code, so let's get our hazmat suits on and jump in!

# An interesting optimization

We have this code here:

```rust
fn takes_box_and_ptr_to_it(mut b: Box<u8>, ptr: *const u8) {
    let value = unsafe { *ptr };
    *b = 5;
    let value2 = unsafe { *ptr };
    assert_ne!(value, value2);
}

let b = Box::new(0);
let ptr: *const u8 = &*b;
    
takes_box_and_ptr_to_it(b, ptr);
```

There's a function, `takes_box_and_ptr_to_it`, that takes a box and a pointer as parameters. Then,
it reads a value from the pointer, writes to the box, and reads a value again. It then asserts that
the two values aren't equal. How can they not be equal? If our box and pointer point to the same
location in memory, writing to the box will cause the pointer to read the new value.

Now construct a box, get a pointer to it, and pass the two to the function. Run the program...

... and everything is fine. Let's run it in release mode. This should work as well, since the optimizer
isn't allowed to change observable behaviour, and an assert is very observable. Run the progrm...

```
thread 'main' panicked at 'assertion failed: `(left != right)`
  left: `0`,
 right: `0`', src/main.rs:5:5
```

Hmm. That's not what I've told would happen. Is the compiler broken? Is this a miscompilation?
I've heard that those do sometimes happen, right?

Trusting our instincts that "it's never a miscompilation until it is one", we assume that LLVM behaved
well here. But what allows it to make this optimization? Taking a look at the generated LLVM-IR (by using
`--emit llvm-ir`) reveals the solution: (severely shortened to only show the relevant parts)

```llvmir
define void @takes_box_and_ptr_to_it(i8* noalias %0, i8* %ptr) {
```

See the little attribute on the first parameter called `noalias`? That's what's doing the magic here.
`noalias` is quite complex, but for our case here, it says that no other pointers point to (alias) the same location.
This allows the optimizer to assume that writing to the box pointer doesn't affect the other pointer - they are
not allowed to alias (it's like if they used `restrict` in C).

If you're a viewer of [Jon Gjengset](https://twitter.com/jonhoo)'s content (which I can highly recommend), 
this might sound familiar to you: Jon has made an entire video about this before, since his crate `left-right`
was affected by this (https://youtu.be/EY7Wi9fV5bk).

If you're looking for _any_ hint that using box emits `noalias`, you have to look no further than the documentation
for [`std::boxed`](https://doc.rust-lang.org/nightly/std/boxed/index.html#considerations-for-unsafe-code). Well, the nightly or beta docs, because I only added this section very recently. For years, this behaviour was simply undocumented. So lots of
code was written thinking that box was "just a RAII pointer" (a pointer that allocates the value in the constructor,
and deallocates it in the destructor on drop) for all pointer are concerned.

# Stacked Borrows and Miri

[https://github.com/rust-lang/miri](Miri) is an interpreter for Rust code with the goal of finding undefinde behaviour.
Undefined behaviour, UB for short, is behaviour of a program upon which no restrictions are imposed. If UB is executed,
_anything_ can happen, including segmentation faults, silent memory corruption, leakage of private keys or exactly
what you intended to happen. Examples of UB include use-after-free, out of bounds reads or data races.

I cannot recommend Miri highly enough for all unsafe code you're writing (sadly support for some IO functions
and FFI is still lacking).

So, let's see whether our code contains UB. It has to, since otherwise the optimizer wouldn't be allowed to change
observable behaviour (since the assert doesn't fail in debug mode).

```rust
error: Undefined Behavior: attempting a read access using <3314> at alloc1722[0x0], but that tag does not exist in the borrow stack for this location
  --> src/main.rs:2:26
   |
2  |     let value = unsafe { *ptr };
   |                          ^^^^
   |                          |
   |                          attempting a read access using <3314> at alloc1722[0x0], but that tag does not exist in the borrow stack for this location
   |                          this error occurs as part of an access at alloc1722[0x0..0x1]
   |
   = help: this indicates a potential bug in the program: it performed an invalid operation, but the Stacked Borrows rules it violated are still experimental
   = help: see https://github.com/rust-lang/unsafe-code-guidelines/blob/master/wip/stacked-borrows.md for further information
help: <3314> was created by a retag at offsets [0x0..0x1]
  --> src/main.rs:10:26
   |
10 |     let ptr: *const u8 = &*b;
   |                          ^^^
help: <3314> was later invalidated at offsets [0x0..0x1]
  --> src/main.rs:12:29
   |
12 |     takes_box_and_ptr_to_it(b, ptr);
   |                             ^
   = note: backtrace:
   = note: inside `takes_box_and_ptr_to_it` at src/main.rs:2:26
note: inside `main` at src/main.rs:12:5
  --> src/main.rs:12:5
   |
12 |     takes_box_and_ptr_to_it(b, ptr);
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

This behaviour does indeed not look very defined at all. But what went wrong? There's a lot of information here.

First of all, it says that we attempted a read access, and that this access failed because the tag does not exist in the
borrow stack. This is something about stacked borrows, the experimental memory model for Rust that is implemented
in Miri. For an excellent introduction, see this part of the great book "Learning Rust With Entirely Too Many Linked Lists:
https://rust-unofficial.github.io/too-many-lists/fifth-stacked-borrows.html.

In short: each pointer has a unique tag attacked to it. Bytes in memory have a stack of such tags, and only the pointer
that have their tag in the stack are allowed to access it. Tags can be pushed and popped onto the stack through various
operations, for example borrowing.