# Include Folder

A simple procedural macro to include the contents of a directory into your project.

This differs from [`include_dir`](https://crates.io/crates/include_dir), because it recursively generates structs for the directory and subdirectories making it easier to access files and so they show up in your IDE's autocomplete.

It first attempts to parse the contents of the file as UTF-8, if it can, it will store this as a
`String`, other wise it will store it as a `Vec<u8>`.

Due to the way files are stored as fields in a struct, your file names must follow the same rules
as a Rust identifier. Eg: cannot start with a number, cannot be a keyword like mod. However they
can, of course, contain '.'.

## Example

Say we have a directory with a structure that looks like this:

```text
src
├── main.rs
├── nested
│   └── folders
│       └── test.txt
└── parsing
    ├── lexer.rs
    └── mod.rs
```

We can access the files in that directory like this:

```rs
use include_folder::include_folder;

// First argument is the path to the directory.
// Second argument is a name for that folder.
//   This crate uses heck under the hood so you can
//   use any case you want, for example: PascalCase.
include_folder!("./src", "src_dir");

fn main() {
    let dir = build_dir();
    let contents: String = dir.nested.folders.test.txt;

    dbg!(contents); // "Hello World!\n"
}
```

Here is the generated code:

```rs
use include_folder::include_folder;

struct BuildDir {
    nested: BuildDirNested,
    parsing: BuildDirParsing,
    main: BuildDirMain,
}
struct BuildDirNested {
    folders: BuildDirNestedFolders,
}
struct BuildDirNestedFolders {
    test: BuildDirNestedFoldersTest,
}
struct BuildDirNestedFoldersTest {
    txt: String,
}
struct BuildDirParsing {
    lexer: BuildDirParsingLexer,
}
struct BuildDirParsingLexer {
    rs: String,
}
struct BuildDirMain {
    rs: String,
}

fn build_dir() -> BuildDir {
    BuildDir {
        nested: BuildDirNested {
            folders: BuildDirNestedFolders {
                test: BuildDirNestedFoldersTest {
                    txt: "Hello World!\n".to_string(),
                },
            },
        },
        parsing: BuildDirParsing {
            lexer: BuildDirParsingLexer {
                rs: "".to_string(),
            },
        },
        main: BuildDirMain { rs: "".to_string() },
    }
}

fn main() {
    let dir = build_dir();
    let contents: String = dir.nested.folders.test.txt;
    // ...
}
```

This works also with multiple files of the same name. Say we added a `lexer.txt` next to the `lexer.rs`, we would get this instead:

```rs
struct BuildDirParsingLexer {
    txt: String,
    rs: String,
}
```
