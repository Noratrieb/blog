+++
title = "Box Is a Unique Type"
date = "2022-07-23"
author = "Noratrieb"
authorTwitter = "@Noratrieb"
tags = ["rust", "unsafe code"]
keywords = ["box", "noalias"]
description = "About better aliasing semantics for `Box<T>`"
showFullContent = false
readingTime = true
hideComments = false
draft = false
+++

We have all used `Box<T>` before in our Rust code. It's a glorious type, with great ergonomics
and flexibility. We can use it to put our values on the heap, but it can do even more
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
isn't allowed to change observable behaviour, and an assert is very observable. Run the program...

```
thread 'main' panicked at 'assertion failed: `(left != right)`
  left: `0`,
 right: `0`', src/main.rs:5:5
```

Hmm. That's not what I've told would happen. Is the compiler broken? Is this a miscompilation?
I've heard that those do sometimes happen, right?

Trusting our instincts that "it's never a miscompilation until it is one", we assume that LLVM behaved
well here. But what allows it to make this optimization? Taking a look at the generated LLVM-IR (by using
`--emit llvm-ir -O`, the `-O` is important since rustc only emits these attributes with optimizations on)
 reveals the solution: (severely shortened to only show the relevant parts)

```llvmir
define void @takes_box_and_ptr_to_it(i8* noalias %0, i8* %ptr) {
```

See the little attribute on the first parameter called `noalias`? That's what's doing the magic here.
`noalias` is an LLVM attribute on pointers that allows for various optimizations. If there are two pointers,
and at least one of them is `noalias`, there are some restrictions around the two. Approximately:
- If one of them writes, they must not point to the same value (alias each other)
- If neither of them writes, they can alias just fine.
Therefore, we also apply `noalias` to `&mut T` and `&T` (if it doesn't contain interior mutability through 
`UnsafeCell<T>`), since they uphold these rules.

For more info on `noalias`, see [LLVMs LangRef](https://llvm.org/docs/LangRef.html#parameter-attributes).

This might sound familiar to you if you're a viewer of [Jon Gjengset](https://twitter.com/jonhoo)'s content (which I can highly recommend). Jon has made an entire video about this before since his crate `left-right`
was affected by this (https://youtu.be/EY7Wi9fV5bk).

If you're looking for _any_ hint that using box emits `noalias`, you have to look no further than the documentation
for [`std::boxed`](https://doc.rust-lang.org/nightly/std/boxed/index.html#considerations-for-unsafe-code). Well, the nightly or beta docs, because I only added this section very recently. For years, this behaviour was not really documented, and you had to
belong to the arcane circles of the select few who were aware of it. So lots of code was written thinking that box was "just an
RAII pointer" (a pointer that allocates the value in the constructor and deallocates it in the destructor on drop) for all
pointers are concerned.

# Stacked Borrows and Miri

So, LLVM was completely correct in optimizing our code to make the assert fail. But what exactly gave it permission to do so?
Undefined Behaviour (UB for short). Undefined behaviour is at the root of many modern compiler optimizations. But what is undefined behaviour?
UB represents a contract between the program and the compiler. The compiler assumes that UB will not happen, and can therefore optimize based
on these assumptions. Examples of UB also include use-after-free, out of bounds reads or data races. If UB is executed, _anything_ can happen,
including segmentation faults, silent memory corruption, leakage of private keys or exactly what you intended to happen.

[Miri](https://github.com/rust-lang/miri) is an interpreter for Rust code with the goal of finding undefined behaviour in Rust. I cannot recommend Miri
highly enough for all unsafe code you're writing (sadly support for some IO functions and FFI is still lacking, and it's still very slow).

So, let's see whether our code contains UB. It has to, since otherwise the optimizer wouldn't be allowed to change
observable behaviour (since the assert doesn't fail in debug mode). `$ cargo miri run`...

```rust,ignore
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
borrow stack of the byte that was accessed. This is something about stacked borrows, the experimental memory model for Rust
that is implemented in Miri. For an excellent introduction, see this part of the great book [Learning Rust With Entirely Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/fifth-stacked-borrows.html).

In short: each pointer has a unique tag attached to it. Each byte in memory has its own 'borrow stack' of these tags,
and only the pointers that have their tag in the stack are allowed to access it. Tags can be pushed and popped from the stack through various operations, for example, borrowing.

In the code example above, we get a nice little hint where the tag was created. When we created a reference (that was then
coerced into a raw pointer) from our box, it got a new tag called `<3314>`. Then, when we moved the box into the function,
something happened: The tag was popped off the borrow stack and therefore invalidated. That's because box invalidates all tags 
when it's moved. The tag was popped off the borrow stack and we tried to read with it anyways - undefined behaviour happened!

And that's how our code wasn't a miscompilation, but undefined behaviour. Quite surprising, isn't it?

# noalias, nothanks

Many people, myself included, don't think that this is a good thing.

First of all, it introduces more UB that could have been defined behaviour instead. This is true for almost all UB, but usually,
there is something gained from the UB that justifies it. We will look at this later. But allowing such behaviour is fairly easy:
If box didn't invalidate pointers on move and instead behaved like a normal raw pointer, the code above would be sound.

But more importantly, this is not behaviour generally expected by users. While it can be argued that box is like a `T`, but on
the heap, and therefore moving it should invalidate pointers, since moving `T` definitely has to invalidate pointers to it,
this comparison doesn't make sense to me. While `Box<T>` usually behaves like a `T`, it's just a pointer. Writers of unsafe
code _know_ that box is just a pointer and will abuse that knowledge, accidentally causing UB with it. While this can be
mitigated with better docs and teaching, like how no one questions the uniqueness of `&mut T` (maybe that's also because that
one makes intuitive sense, "shared xor mutable" is a simple concept), I think it will always be a problem,
because in my opinion, box being unique and invalidating pointers on move is simply not intuitive.

When a box is moved, the pointer bytes change their location in memory. But the bytes the box points to stay the same. They don't
move in memory. This is the fundamental missing intuition about the box behaviour.

There are also other reasons why the box behaviour is not desirable. Even people who know about the behaviour of box will want
to write code that goes directly against this behaviour at some point. But usually, fixing it is pretty simple: Storing a raw
pointer (or `NonNull<T>`) instead of a box, and using the constructor and drop to allocate and deallocate the backing box.
This is fairly inconvenient but totally acceptable. There are bigger problems though. There are crates like `owning_ref`
that want to expose a generic interface over any type. Users like to choose box, and sometimes _have_ to choose box because of
other box-exclusive features it offers. Even worse is `string_cache`, which is extremely hard to fix.

Then last but not least, there's the opinionated fact that `Box<T>` shall be implementable entirely in user code. While we are
many missing language features away from this being the case, the `noalias` case is also magic descended upon box itself, with no
user code ever having access to it.

There are several arguments in favour of box being unique and special cased here. To negate the last argument above, it can
be said that `Box<T>` _is_ a very special type. It's just like a `T`, but on the heap. Using this mental model, it's very easy to
justify all the box magic and its unique behaviour. But in my opinion, this is not a useful mental model regarding unsafe code,
and I prefer the mental model of "reference that manages its own lifetime", which doesn't imply uniqueness.

But there are also crates on [crates.io](https://crates.io/) like [aliasable](https://crates.io/crates/aliasable) that already
provide an aliasable version of `Box<T>`, which is used by the self-referential type helper crate [ouroboros](https://crates.io/crates/ouroboros).
So if box stayed unique, people could also just pick up that crate as a dependency and use the aliasable box from there instead of
having to write their own. Interestingly, this crate also provides a `Vec<T>`, even though `Vec<T>` can currently be aliased in practice and
in the current version of stacked borrows just fine, although it's also not clear whether we want to keep it like this, but I
don't think this can reasonable be changed.

> One thing was just pointed out to me after releasing the post: Mutation usually goes through `&mut T` anyways, even when the value
> is stored as a `Box<T>`. Therefore, all the guarantees of uniqueness are already present when mutating boxes, making the uniqueness
> of box even less important.

# noalias, noslow

There is one clear potential benefit from this box behaviour: ✨Optimizations✨. `noalias` doesn't exist for fun, it's something
that can bring clear performance wins (for `noalias` on `&mut T`, those were   measureable). So the only question remains:
**How much performance does `noalias` on `Box<T>` give us now, and how many potential performance improvements could we get in the 
future?** For the latter, there is no simple answer. For the former, there is. `rustc` has [_no_ performance improvements](https://github.com/rust-lang/rust/pull/99527) 
from being compiled with `noalias` on `Box<T>`, but this isn't really representative since rustc mostly uses arenas instead of box internally.

I have also benchmarked a few crates from the ecosystem with and without noalias on box, and the [results](https://gist.github.com/Noratrieb/9a0751fb9fd1044a30ab55cef9a7d335)
were inconclusive. (At the time of writing, only regex-syntax, tokio, and syn have been benchmarked.) regex-syntax showed no changes. Tokio showed a few improvements without noalias
which is very weird, so maybe the benchmarks aren't really good or something else was going on. And syn tended towards minor regressions without noalias, but the benchmarks had high
jitter so no real conclusion can be reached from this either, at least in my eyes, but I don't have a lot of experience with benchmarks. Therefore, I would love for more people
to benchmark more crates, especially if you have more experience with benchmarks.

# a way forward

Based on all of this, I do have a few solutions. First of all, I think that even if there might be some small performance regressions, they are not significant enough
to justify boxes uniqueness. Unsafe code wants to use box, and it is reasonable to do so. Therefore I propose to completely remove all uniqueness from `Box<T>` and treat it
just like a `*const T` for aliasing. This will make it more predictable for unsafe code and is a step forward towards less magic from `Box<T>`.

But the performance cost may be real, and especially the future optimization value can't be certain. The current uniqueness guarantees of box
are very strong, and still giving code an option to obtain these seems useful. One possibility would be for code to use a 
`&'static mut T` that is unleaked for drop, but the semantics of this are still [unclear](https://github.com/rust-lang/unsafe-code-guidelines/issues/316).
If that is not possible, exposing `std::ptr::Unique` (with it getting boxes aliasing semantics) could be desirable. For this, all existing usages of `Unique` 
inside the standard library would have to be removed. We could also offer a `std::boxed::UniqueBox` that keeps the current semantics, but this would also bring direct aliasing
decisions more towards safe code, which I am not a huge fan of. Ownership is enough already.

I guess what I am wishing for are some good and flexible raw pointer types. But that's still in the stars...

For more information about this topic, see https://github.com/rust-lang/unsafe-code-guidelines/issues/326

_Thanks to the nice people on the Rust Community Discord for their feedback on the draft of this post!_
