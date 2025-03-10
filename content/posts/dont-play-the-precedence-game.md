+++
title = "Don't play the precedence game"
date = "2025-03-10"
author = "Noratrieb"
tags = ["language-design"]
keywords = ["language", "design"]
description = "Why programming languages should abolish most operator precedence"
showFullContent = false
readingTime = true
hideComments = false
draft = false
+++

If you've ever done any math, you are familiar with operator precedence, even if you don't know that word.
The result of the mathematical expression `2 + 1 * 3` is 5 and not 9, because the right multiplication expression is evaluated first, so we get `2 + (1 * 3)`.
The mathematical ordering for this is exponents, multiplication/division, and then addition/subtraction (sometimes known as "PEMDAS").

Programming languages take this much further. They add many new operators into the mix (for example, bit manipulation or comparison), and all of these operators fit into this hierarchy.

In C, it's as follows (from [the standard](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n3220.pdf)):
- multiplicative (`*`, `/`, `%`)
- additive (`+`, `-`)
- bit shifts (`<<`, `>>`)
- comparison (but not equality) (`<`, `>`, `<=`, `>=`)
- equality (`==`, `!=`)
- bit and (`&`)
- bit xor (`^`)
- bit or (`|`)
- logical and (`&&`)
- logical or (`||`)

If I asked you to recite this order, you would probably get it wrong.
It's completely arbitrary and impossible to remember unless you really practice it.

But it gets worse.

Let's look at Rust (from [the reference](https://doc.rust-lang.org/stable/reference/expressions.html))
- multiplicative (`*`, `/`, `%`)
- additive (`+`, `-`)
- bit shifts (`<<`, `>>`, `>>>`)
- bit and (`&`)
- bit xor (`^`)
- bit or (`|`)
- comparison (`<`, `>`, `<=`, `>=`, `==`, `!=`)
- logical and (`&&`)
- logical or (`||`)

It's different! Rusts choice here arguably makes sense; you can now write `1 & 0 == 0` and it does what you want, but due to the differences between languages it's now gotten even more impossible to remember.

If you ever mix the precedence up, your code will be incorrect. And if a reader mixes them up, they will be very confused why the code is seemingly incorrect even when it is correct.

It's like a game. And the only way to win this game is to **not play**.

Enter: parentheses. We haven't talked about them before here, but they're the "P" in PEMDAS and way above multiplicative in either language.
You can use them to group operators to do exactly what you want, in an obvious way. `(1 & 0) == 0` is correct in C and Rust, and every reader knows what exactly is up.

While it can be a bit verbose, it makes the code much easier to understand without knowing the complex precedence hierarchies.

Which is where this turns into a language design post: programming languages should not have these hierarchies in the first place, and parentheses should just be required.
It seems acceptable to allow it for the basic math operations most people are familiar with, but there is no reason why `||` and `^` should have a precedence relationship.
There are also some other cases where you might want to have precedence; for example, writing `x > 0 && x > 5` is fairly clear and useful. But in general, not everything should have a relative precedence with everything else.

And until programming languages require you to do this[^lisp], we can at least do it ourselves. And maybe even enable a linter rule that requires it, if it exists for the language.

[^lisp]: LISP already does this in a way by having prefix operator syntax; you do `(add 2 (multiply 1 3))`, always adding parentheses.
