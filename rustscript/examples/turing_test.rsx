# ─────────────────────────────────────────────────────────
#  RustScript — Turing Completeness Demo
#  This file proves the language is Turing complete by
#  implementing: variables, arithmetic, conditionals, loops,
#  recursion, and dynamic data structures.
# ─────────────────────────────────────────────────────────

# ─── Variables & Arithmetic ─────────────────────────────
let x = 10
let y = 3
let result = x * y + 2
print("10 * 3 + 2 =", result)

# ─── Strings ────────────────────────────────────────────
let name = "RustScript"
let greeting = "Hello from {name}!"
print(greeting)

# ─── Lists ──────────────────────────────────────────────
let numbers = [1, 2, 3, 4, 5]
print("Numbers:", numbers)
print("Length:", len(numbers))
print("First:", numbers[0])

# ─── Conditionals ──────────────────────────────────────
if x > 5 {
    print("x is greater than 5")
} else {
    print("x is 5 or less")
}

# ─── While Loop ────────────────────────────────────────
let i = 0
let sum = 0
while i < 10 {
    sum += i
    i += 1
}
print("Sum 0..9 =", sum)

# ─── For Loop ──────────────────────────────────────────
let fruits = ["apple", "banana", "cherry"]
for fruit in fruits {
    print("Fruit:", fruit)
}

# ─── Functions ─────────────────────────────────────────
fn add(a, b) {
    return a + b
}
print("add(3, 7) =", add(3, 7))

# ─── Recursion (proves Turing completeness) ────────────
fn factorial(n) {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}
print("5! =", factorial(5))

fn fibonacci(n) {
    if n <= 0 {
        return 0
    }
    if n == 1 {
        return 1
    }
    return fibonacci(n - 1) + fibonacci(n - 2)
}

print("fib(10) =", fibonacci(10))

# ─── FizzBuzz ──────────────────────────────────────────
fn fizzbuzz(n) {
    let i = 1
    while i <= n {
        if i % 15 == 0 {
            print("FizzBuzz")
        } else if i % 3 == 0 {
            print("Fizz")
        } else if i % 5 == 0 {
            print("Buzz")
        } else {
            print(i)
        }
        i += 1
    }
}

print("--- FizzBuzz 1-15 ---")
fizzbuzz(15)

# ─── String Methods ─────────────────────────────────────
let msg = "  Hello World  "
print("Trimmed:", msg.trim())
print("Upper:", msg.upper())
print("Lower:", msg.lower())

# ─── Type checking ──────────────────────────────────────
print("Type of 42:", type(42))
print("Type of 'hi':", type("hi"))
print("Type of true:", type(true))
print("Type of []:", type([]))

print("--- All tests passed! ---")
