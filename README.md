<div align="center">
    <img src="assets/logo-dark.png#gh-light-mode-only" alt="Nib" width="250">
    <img src="assets/logo-light.png#gh-dark-mode-only" alt="Nib" width="250">
    <br/><br/>
    <a href="LICENSE"><img src="https://img.shields.io/github/license/wyteroze/nib-lang"/></a>
    <a><img src="https://img.shields.io/badge/status-toy--language-blue"/></a>
</div>
<br>
__Nib is a small toy language with a lexer, parser, compiler, and bytecode VM, all written in Rust from scratch, by hand!__ 
You are free to tour the source code, and I've tried to explain confusing concepts when possible.

Nib's a personal project for learning and experimenting with language design, so don't expect anything serious from it.

Nib is unfortunately not 100% halal because of the usage of Rust. (/j but not really)

## Example
```
func fizzbuzz(i) {
    if i % 3 == 0 and i % 5 == 0 {
        return "FizzBuzz"
    } else if i % 3 == 0 {
        return "Fizz"
    } else if i % 5 == 0 {
        return "Buzz"
    } else {
        return i
    }
    
    i = i + 1
}
```

## Status
Supports variables and constants, if/else/else if, arithmetic and comparison operators, strings, and functions.

Not supported (yet (or ever)): structs/tables, closures, a standard library, and a module system.

## License
Nib is licensed under the Do What The Fuck You Want To Public License Version 2. 
The name is self explanatory, but you can read the [LICENSE](LICENSE) for more info.
