# Caedan

Caedan is yet another Brainfuck derivative. It adds a number of features, with the most notable being that of distinct regions on which operations can be applied.

## General Syntax

A Caedan source file (.cae) is a series of declarations, either declarations of procedures or regions. A region declaration is of the form

`region <name>[<size>]`

Where `<name>` is a series of alphanumeric ascii characters (including underscore), and `<size>` is a non-zero decimal number. A procedure declaration is of the form

`proc <name>: <instructions>`

Where `<name>` follows the same form as for regions, and instructions is a set of instructions in the extended syntax devised for the language. Any valid Brainfuck code can be placed in the instructions.

Execution begins at the `main` procedure, on the `main` region. All procedures must be executed on some defined region. A simple hello world (using the Wikipedia example) could be as follows

```cae
region main[100]
proc main: ++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
```

All overflow behaviors are defined to wrap. This includes both increments and decrements, and moving past the boundaries of regions.

## Extensions to the Brianfuck Language

A number of new features were added to the instruction set to make the language easier to work with.

### 1: Reset

The `~` instruction resets the position of the read/write head in the current region to zero.

### 2: Quote

The `"xx` instruction is a quote followed by two hex digits, upper or lower case. The byte defined by the hex digits is written to the position under the read/write head in the current region. Note that there must be two hex digits, even for small numbers.

### 3: Send/Receive

The `^<region>` and `&<region>` instructions allow communication between regions. The first, `^<region>`, sets the byte under the read/write head in the specified region to the byte under the read/write head in the current region. The `&<region>` instruction does the opposite, receiving a byte from the specified region.

### 4: Call

Procedures can be called in three distinct ways: `<procedure>`, `<procedure>@<region>`, and another which will be explained in section 6. These two, at least, are fairly self explanatory. The first executes the given procedure in the current region, and the second executes the given procedure in the given region.

### 5: Anonymous Procedures

Anonymous procedures can be created with round brackets, with the form `(<instructions>)`. An anonymous procedure has exactly the same syntactic rules as a normal procedure, so the call syntax above works as expected. This means that enclosing some instructions in round brackets has no effect, since they will implicitly act on the region they were created in (with the notable exception of square brackets, which must be matched within a procedure. A procedure of the form `proc bad: ([)]` is forbidden).

### 6: Back References

Because an anonymous procedure can be called on a foreign region, one can lose the ability to reference the region that the base procedure was called on. As an example

```cae
region foreign[1]
proc very_sad: "3A."28.
proc example: (very_sad)@foreign
```

Inside of the round brackets, it is impossible for `very_sad` to act on the region that the procedure was invoked for, except by name which limits generality. For this reason, the back reference was introduced. It takes the form `<procedure>$`, and simply means to act on the region that the containing procedure was invoked for. So

```cae
region foreign[1]
proc very_happy: "3A."29.
proc blah: (very_happy$)@foreign
```

The bytes in `very_happy` will always be written to the region the procedure was invoked for. This enables much easier constructs around loops. As an example

```cae
region loop_flag[1]
proc do_stuff: "00
proc loop: ("01[do_stuff$])@loop_flag
```

The `do_stuff` procedure could be anything. It will execute repeatedly until the loop flag is set back to 0.
