# ─────────────────────────────────────────────────────────
#  RustScript v1.1 — New Features Showcase
#  Demonstrates every new language feature added.
# ─────────────────────────────────────────────────────────

/* Multi-line block comments are now supported!
   They can span multiple lines and even be /* nested */ */

# ─── None Literal ───────────────────────────────────────
let x = none
print("none value:", x)
print("none is falsy:", not x)

# ─── Power Operator ** ──────────────────────────────────
print("2 ** 10 =", 2 ** 10)
print("3 ** 3 =", 3 ** 3)

# ─── Floor Division // ─────────────────────────────────
print("17 // 3 =", 17 // 3)
print("10 // 4 =", 10 // 4)

# ─── Compound Assignments *= /= ─────────────────────────
let val = 6
val *= 7
print("6 *= 7 →", val)
val /= 2
print("/= 2 →", val)

# ─── Elif Keyword ──────────────────────────────────────
let score = 85
if score >= 90 {
    print("Grade: A")
} elif score >= 80 {
    print("Grade: B")
} elif score >= 70 {
    print("Grade: C")
} else {
    print("Grade: F")
}

# ─── Break & Continue ──────────────────────────────────
let found = -1
for i in range(100) {
    if i * i > 50 {
        found = i
        break
    }
}
print("First i where i² > 50:", found)

let evens_sum = 0
for i in range(1, 11) {
    if i % 2 != 0 {
        continue
    }
    evens_sum += i
}
print("Sum of evens 1-10:", evens_sum)

# ─── Negative Indexing ──────────────────────────────────
let colors = ["red", "green", "blue", "yellow"]
print("Last color:", colors[-1])
print("Second to last:", colors[-2])

let word = "RustScript"
print("Last char:", word[-1])

# ─── Dictionary Literals ───────────────────────────────
let person = {"name": "Ada", "age": 36, "lang": "RustScript"}
print("Person:", person)
print("Name:", person["name"])
print("Age via dot:", person.age)

# Dict assignment
person["email"] = "ada@example.com"
person.active = true
print("Updated:", person)

# Dict methods
print("Keys:", keys(person))
print("Values:", values(person))
print("Has name?", has(person, "name"))
print("Get missing:", person.get("phone", "N/A"))

# ─── Range with Step ───────────────────────────────────
print("Count by 2s:", range(0, 10, 2))
print("Countdown:", range(10, 0, -1))

# ─── Lambda Expressions ────────────────────────────────
# Rust-inspired |params| syntax
let double = |x| x * 2
let add = |a, b| a + b
print("double(5):", double)  # lambdas display as <lambda>
print("Lambda type:", type(double))

# ─── Pipe Operator |> ──────────────────────────────────
# Elixir/F#-inspired — pass result as first argument
let nums = [3, 1, 4, 1, 5, 9, 2, 6]
let result = nums |> sorted
print("Sorted:", result)
let rev = nums |> reversed
print("Reversed:", rev)

# ─── Map / Filter / Reduce ─────────────────────────────
let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

let squares = map(numbers, |x| x ** 2)
print("Squares:", squares)

let odds = filter(numbers, |x| x % 2 != 0)
print("Odds:", odds)

let total = reduce(numbers, 0, |acc, x| acc + x)
print("Sum via reduce:", total)

# ─── List Methods ──────────────────────────────────────
let items = [5, 3, 8, 1, 9, 2, 8]
print("Sorted:", items.sort())
print("Reversed:", items.reverse())
print("Unique:", items.unique())
print("First:", items.first())
print("Last:", items.last())
print("Slice [1:4]:", items.slice(1, 4))
print("Contains 8?", items.contains(8))
print("Index of 3:", items.index(3))
print("Count of 8:", items.count(8))
let nested = [[1, 2], [3, 4], [5]]
print("Flat:", nested.flat())

# Method chaining with lambdas
let processed = items.filter(|x| x > 3).sort()
print("Filtered & sorted:", processed)

# ─── String Methods ────────────────────────────────────
let msg = "  Hello, World!  "
print("Strip:", msg.strip())
print("Upper:", msg.upper())
print("Lower:", msg.lower())
print("Replace:", msg.replace("World", "RustScript"))
print("Starts with space?", msg.starts_with(" "))
print("Ends with !  ?", msg.ends_with("!  "))
print("Find 'World':", msg.find("World"))
print("Count 'l':", msg.count("l"))
print("Chars:", "abc".chars())
print("Is digit?", "123".is_digit())
print("Is alpha?", "abc".is_alpha())
print("Slice:", "RustScript".slice(0, 4))
print("Repeat:", "ha".repeat(3))
print("Split:", "a,b,c".split(","))

# ─── New Built-in Functions ────────────────────────────
print("sum:", sum([1, 2, 3, 4, 5]))
print("round:", round(3.14159, 2))
print("chr(65):", chr(65))
print("ord('A'):", ord("A"))
print("sorted:", sorted([5, 2, 8, 1]))
print("reversed:", reversed([1, 2, 3]))
print("enumerate:", enumerate(["a", "b", "c"]))
print("zip:", zip([1, 2, 3], ["a", "b", "c"]))
print("any:", any([false, false, true]))
print("all:", all([true, true, true]))
print("bool(0):", bool(0))
print("bool(1):", bool(1))

# ─── Dict Iteration ────────────────────────────────────
let scores = {"alice": 95, "bob": 87, "carol": 92}
for name in scores {
    print("Student:", name)
}
for pair in items(scores) {
    print("  →", pair)
}

# ─── Assert ────────────────────────────────────────────
assert(2 + 2 == 4, "Math is broken!")
assert(true, "This should pass")
print("All assertions passed!")

print("\n── RustScript v2 showcase complete! ──")
