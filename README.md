# lithp
Like Lisp, but the same

## Description

This is a small Lisp similar to Scheme, except missing some features such as continuations.

It is also more pure than lisp, and doesn't allow modification of bindings after they are made, meaning
that most programs must be contained in a master "let" block, and recusion is only possible by using the
built-in function "recur".

Ideally this would be modified to use continuation-passing / trampoline-style evaluation.
