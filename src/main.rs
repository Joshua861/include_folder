use include_folder::include_folder;

include_folder!("include_folder_macros/test_folder/src", "BuildDir");

fn main() {
    let dir = build_dir();
    let contents: String = dir.nested.folders.test.txt;

    dbg!(contents); // "Hello World!\n"
}
