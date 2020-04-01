# Quick guide for write grammars

Hi!

You want to write a grammar quickly

Let's begin

# Rules

The rules are the way to define a sentence

```rust
// name = {"something"}
organization = {"IETF means Internet Engineering Task Force"}
```

A rule can refer to more rules

```rust
browser = {"Firefox"}
fact = {browser ~ " has a good performance."}

// prints: "Firefox has a good performance."
```

# Ordered choices

You can have multiple options with "|"

```rust
browser = {"Firefox" | "Chrome" | "Safari"}
fact = {browser ~ " has a good support."}

// "Firefox has a good support."
// "Chrome has a good support."
// "Safari has a good support."
```

# Options

You can generate something or not... with "?"

```rust
evasion = "don't"
sentence = {"I " ~ evasion? ~ " like to clean my room."}

// {"I like to clean my room."}
// {"I don't like to clean my room."}
```

# Repetitions

You can repeat a rule n times with `{n}`

```rust
uncontrollable = {" great"}
fact = {"I have a" ~ uncontrollable{3} ~ " desire to eat hamburger."}

// "I have a great great great desire to eat hamburger."
```

You can repeat a rule in a range with `{n,m}` [inclusive, inclusive]

```rust
uncontrollable = {" great"}
fact = {"I have a" ~ uncontrollable{2,4} ~ " desire to eat hamburger."}

// "I have a great great desire to eat hamburger."
// "I have a great great great desire to eat hamburger."
// "I have a great great great great desire to eat hamburger."
```

You can repeat a rule at most n times with `{,n}`, it's like {0, n}

```rust
uncontrollable = {" great"}
fact = {"I have a" ~ uncontrollable{,3} ~ " desire to eat hamburger."}

// "I have a desire to eat hamburger."
// "I have a great desire to eat hamburger."
// "I have a great great to eat hamburger."
// "I have a great great great desire to eat hamburger."
```

You can repeat a rule at least n times with `{n,}`, it's like {n, infinity} (Not really, but there are an option to control the upper limit)

```rust
uncontrollable = {" great"}
fact = {"I have a" ~ uncontrollable{,3} ~ " desire to eat hamburger."}

// "I have a desire to eat hamburger."
// "I have a great desire to eat hamburger."
// "I have a great great to eat hamburger."
// "I have a great great great desire to eat hamburger."
```

You can repeat a rule zero or more times with `*`, it's like {0, infinity} (Not really, but there are an option to control the upper limit)

```rust
uncontrollable = {" great"}
fact = {"I have a" ~ uncontrollable* ~ " desire to eat hamburger."}

// "I have a desire to eat hamburger."
// "I have a great desire to eat hamburger."
// ...
// "I have a great great great great desire to eat hamburger."
```

You can repeat a rule one or more times with `*`, it's like {1, infinity} (Not really, but there are an option to control the upper limit)

```rust
uncontrollable = {" great"}
fact = {"I have a" ~ uncontrollable+ ~ " desire to eat hamburger."}

// "I have a great desire to eat hamburger."
// ...
// "I have a great great great great desire to eat hamburger."
```

# Grouping

You can group things with parentheses "(" and ")"

```rust
// aerobics will have a probability of 0.25 to be generated.
// archery will have a probability of 0.25 to be generated.
// cycling will have a probability of 0.5 to be generated.
sport = {("aerobics" | "archery") | "cycling"}
sentence = {"I like to do " ~ sport ~ " in the mornings."}
```

# The negation predicate

You can avoid the generation of an element with this pattern

```rust
// Or letter = {ASCII_ALPHA_LOWER}
letter = { "a" | "b" | "c" | "d" | "e" | "f" | "g" }
vocal = { "a" | "e" | "i" | "o" | "u"}

consonant = {!vocal ~ letter}
```

The only constraint to use this is that the exclamation point must be accompanied by a rule

```rust
// This is valid, but doesn't work

consonant = {!("a" | "e" | "i" | "o" | "u") ~ letter}
```

# Char ranges

You can define a range of characters like this:

```rust
letter = { 'a'..'z' }
other = {'b'..'m'}
hex = {'A'..'F'}
```

# Built-in rules

There are rules available for simplify the grammar

**Note:** Currently only the ascii rules from pest reference are supported.

| Rule                | Equivalent                                      |
| ------------------- | ----------------------------------------------- |
| ANY                 | `('\u{00}'..'\u{10FFFF}')`                      |
| ASCII_DIGIT         | `('0'..'9')`                                    |
| ASCII_NONZERO_DIGIT | `('1'..'9')`                                    |
| ASCII_BIN_DIGIT     | `('0'..'1')`                                    |
| ASCII_OCT_DIGIT     | `('0'..'7')`                                    |
| ASCII_HEX_DIGIT     | <code>`('0'..'9' | 'a'..'f' | 'A'..'F')`</code> |
| ASCII_ALPHA_LOWER   | `('a'..'z')`                                    |
| ASCII_ALPHA_UPPER   | `('A'..'Z')`                                    |
| ASCII_ALPHANUMERIC  | `('0'..'9' 'a'..'z' \| 'A'..'Z')`               |
| NEWLINE             | <code>`("\n" | "\r\n" | "\r")`</code>           |

```rust
number = {ASCII_DIGIT}

// Some day
myWage = ASCII_NONZERO_DIGIT{5,10}

secretMessage = ASCII_ALPHANUMERIC+
```
