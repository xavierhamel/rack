Rack is a *stackbased* programming language inspired by Forth, every operation push or pop on the stack. Because the language is stackbased and for a very simple parsing, the language uses the reverse polish notation (`2 3 +` instead of `2 + 3`). For easier reading of a codebase, some functionnality can be written without this notation (like function argument) but this is not required.

This language is simply a toy but is fully fleged, if you want to do something there is a way to do it. The standard library implement some default feature documented bellow, if more are necessery you must develop them yourself. Rack will never be optimised, it is outside the scope of this project. Implementing rack with LLVM (or an alternative) is not really an option because of the paradigm of the language. It could be done but would remove the reason why this paradigm was choosen (easier compiler devlopment).

Rack only support windows because it uses the windows api.

## Compiling
To compile a rack file, use the next command to generate assembly:
```
rack.exe <file_to_compile>
```
And you will also need to compile and link the assembly file:
```
nasm -f win64 output.asm -o output.obj & link output.obj /subsystem:console /entry:_start /out:output.exe kernel32.lib
```
And finally, run the output:
```
.\output.exe
```

## Hello world
You can write the Hello world with a common syntax like this:
```
std::println_str("Hello, world!")
```
But this will be changed internally to something like this before being compiled:
```
"Hello, world!" std::println_str
```
This change is done because rack uses the reverse polish notation. You can pass arguments to functions ubt arguments of functions are only the top most values on the stack, therefore you can also just push the arguments on the stack.

## Types (all immutable)
`str`: Static c-string like. They are written with double quotes.

`char`: One character, represented internally as a number. They are written with single quotes.

`int`: 64 bits integer.

`ptr`: 64 bits integer pointing to an other value.

## Intrinsics
### Memory operations
`mem`: Push a pointer to an internal memory buffer of 256 bytes that can be used freely. For more memory, see `std::alloc`

`dup`: Duplicate and push the top most value on the stack.

`over`: Duplicate and push the second value on the stack.

`drop`: Drop (pop) the topmost value of the stack.

`swap`: Swap the 2 top most values of the stack.

`rot`: Bring the 3rd value of the stack to the top.

`<identifier> put`: Save a value to a variable named `<identifier>` (pop the last value off the stack).

`<identifier> fetch` or `<identifier>!`: Push the value of the variable on the stack.

### Arithmetic operations
`+`, `-`, `*`, `/`, `%`: Pop the 2 top most values off the stack and push back the result.

`&`, `|`: Pop the 2 top most values off the stack and push back the result of a binary or and binary and. Can also be used as a logical and and logical or.

### Comparison operations
`=`, `!=`, `>`, `>=`, `<`, `<=`: Pop the 2 top most values off the stack and push 1 if true or 0 if false

### Control flow
Execute the `<body>` while the condition is still true.
```
while <condition> do 
    <body>
end
```
Execute the `<true_body>` if the condition is true and execute the `<false_body>` if the condition is false.
```
<condition> if
    <true_body>
else
    <false_body>
end
```
Save a constant to be used (without a fetch instruction). The `<value>` can only be an integer.
```
const <identifier> <value> end
```
A function that can be called. Arguments are passed on the stack. Arguments type annotation is mandatory and a check is being done.
```
fn <identifier>[<args_types> -> <return_types>]
    <body>
end
```

### System calls (with the windows api)
Before using a windows api function, you must declare the number of argument that the function uses as a const
```
const WriteConsoleA 4 end
const GetStdHandler 1 end
```
You can now call the `WriteConsoleA` windows function by appending `sys::` before the function name:
```
sys::WriteConsoleA(sys::GetStdHandler(-11), "Hello", 5, mem)
```
In this example, `WriteConsoleA` will print to the standard output "Hello".

## Some standard functions
### `str::find_char[int, str|ptr -> int]`
Will find the first occurence of a char in a `str` and return the index of the char (or -1 if the char was not found).
Usage : `str::find_char(<char>, <string>)`
### `str::find_char_from[int, int, str|ptr -> int]`
Will find the first occurence of a char in a `str` starting at the n position and return the index of the char (or -1 if the char was not found).
Usage : `str::find_char(<char>, <from>, <string>)`
### `str::len[str|ptr -> int]`
Return the length of a string.
Usage: `str::len(<string>)`
### `str::print[str|ptr -> void]`
Print a string to the stdout on the current line. Alias for `std::print_str`
Usage: `str::print(<string>)`
### `std::exit[void]`
Exit the current program with an exit code 0.
Usage: `std::exit`
### `std::assert[int, int, str|ptr -> void]`
Assert that the 2 values are equal, if not print the string and exit the program.
Usage: `std::assert(0, 0, "0 equals to 0")` 
### `std::print_str[str|ptr -> void]`
Print a string to the stdout on the current line. Alias for `str::print`
Usage: `std::print_str(<string>)`
### `std::print_int[int -> void]`
Print an integer to the stdout on the current line and add a new line after.
Usage: `std::print_int(<string>)`
### `std::throw[str|ptr -> void]`
Print the given message and exit the program with error code 0.
Usage: `std::throw(<string>)`
### `std::alloc[int -> ptr]`
Allocate the specifed size on the heap and return a pointer to the memory allocated.
Usage: `std::alloc(<size>)`
### `std::realloc[ptr, int -> ptr]`
Reallocate a memory block with bigger capacity and return the new address of the pointer.
Usage: `std::realloc(<pointer>, <new_size>)`
### `std::free[ptr -> void]`
Free an allocated memory block.
Usage: `std::free(<pointer>)`

### More functions
For more functions and better documentation, directly read `/tests/std.rk`, `/tests/string.rk`, `/tests/str.rk` and `/tests/vec.rk`. They are also good examples of how to write a program in rack.

## Rack binary usage
### Compiling a program
use `rack.exe <file_to_compile>`

### Debug the stack
This helps when you want to visualize the stack of a function and see what each operation really does:
`rack.exe <file_to_compile> --debug-stack <function_name_to_debug>`