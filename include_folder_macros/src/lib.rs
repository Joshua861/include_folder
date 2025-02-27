use std::{collections::HashMap, fs, path::Path};

use anyhow::Result;
use heck::{ToPascalCase, ToSnekCase};
// use image::DynamicImage;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    LitStr, Token,
};

#[derive(Debug, Clone)]
enum File {
    Blob(Vec<u8>),
    // Image(DynamicImage),
    Text(String),
}

impl File {
    fn _type(&self) -> String {
        match self {
            Self::Blob(_) => "Vec<u8>",
            Self::Text(_) => "String",
        }
        .to_string()
    }
}

#[derive(Debug, Clone)]
enum Tree {
    Leaf(File),
    Branch(HashMap<String, Tree>),
}

struct Input {
    path: String,
    name: String,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = input.parse::<LitStr>()?.value();

        let _ = input.parse::<Token![,]>()?;

        let name = input.parse::<LitStr>()?.value();

        Ok(Input { path, name })
    }
}

#[proc_macro]
pub fn include_folder(tokens: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokens as Input);

    let tree = match build_tree(&input.path) {
        Ok(tree) => tree,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to build directory tree: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    let tree = process_tree(tree);

    let mut top = quote! {};
    let inner = gen_code(tree, &mut top, input.name.to_pascal_case());
    let function_name = syn::Ident::new(&input.name.to_snek_case(), proc_macro2::Span::call_site());
    let return_type = syn::Ident::new(&input.name.to_pascal_case(), proc_macro2::Span::call_site());

    let output = quote! {
        #top

        fn #function_name () -> #return_type {
            #inner
        }
    };

    dbg!(&output.to_string());

    output.into()
}

fn build_tree(dir_path: &str) -> Result<Tree> {
    let path = Path::new(dir_path);
    if !path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", dir_path));
    }

    if path.is_file() {
        let file = read_file(path)?;
        return Ok(Tree::Leaf(file));
    }

    let mut dir_map = HashMap::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(file_name) = path.file_name() {
            if file_name.to_string_lossy().starts_with('.') {
                continue;
            }
        }

        let file_name = path.file_name().unwrap().to_string_lossy().to_string();

        if path.is_file() {
            match read_file(&path) {
                Ok(file) => {
                    dir_map.insert(file_name, Tree::Leaf(file));
                }
                Err(e) => {
                    eprintln!("Error reading file {}: {}", path.display(), e);
                    continue;
                }
            }
        } else if path.is_dir() {
            match build_tree(path.to_str().unwrap()) {
                Ok(branch) => {
                    dir_map.insert(file_name, branch);
                }
                Err(e) => {
                    eprintln!("Error processing directory {}: {}", path.display(), e);
                    continue;
                }
            }
        }
    }

    Ok(Tree::Branch(dir_map))
}

fn read_file(path: &Path) -> Result<File> {
    /*let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    if extension == "png" || extension == "jpg" || extension == "jpeg" || extension == "gif" {
        let img = image::open(path)?;
        Ok(File::Image(img))
    } else*/
    {
        let content = fs::read(path)?;

        match String::from_utf8(content.clone()) {
            Ok(text) => Ok(File::Text(text)),
            Err(_) => Ok(File::Blob(content)),
        }
    }
}

fn process_tree(tree: Tree) -> Tree {
    match tree {
        Tree::Branch(map) => {
            let mut new_map = HashMap::new();

            for (k, v) in map {
                match v {
                    Tree::Leaf(file) => {
                        let parts: Vec<String> = k
                            .split('.')
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string())
                            .collect();

                        if parts.is_empty() {
                            new_map.insert(k, Tree::Leaf(file));
                        } else {
                            merge_into_map(&mut new_map, &parts, file);
                        }
                    }
                    Tree::Branch(inner_map) => {
                        new_map.insert(k, process_tree(Tree::Branch(inner_map)));
                    }
                }
            }

            Tree::Branch(new_map)
        }
        Tree::Leaf(file) => Tree::Leaf(file),
    }
}

fn merge_into_map(map: &mut HashMap<String, Tree>, parts: &[String], file: File) {
    if parts.is_empty() {
        return;
    }

    let current_part = &parts[0];

    if parts.len() == 1 {
        map.insert(current_part.clone(), Tree::Leaf(file));
        return;
    }

    if !map.contains_key(current_part) {
        map.insert(current_part.clone(), Tree::Branch(HashMap::new()));
    }

    if let Some(Tree::Branch(ref mut next_map)) = map.get_mut(current_part) {
        merge_into_map(next_map, &parts[1..], file);
    } else {
        let mut next_map = HashMap::new();
        merge_into_map(&mut next_map, &parts[1..], file);
        map.insert(current_part.clone(), Tree::Branch(next_map));
    }
}

fn gen_code(tree: Tree, top: &mut TokenStream2, path: String) -> TokenStream2 {
    let path_ident = syn::Ident::new(&path, proc_macro2::Span::call_site());

    match tree {
        Tree::Leaf(file) => match file {
            File::Blob(data) => {
                let iter = data.into_iter();
                quote! { vec![ #(#iter),* ] }
            }
            File::Text(text) => {
                quote! { #text.to_string() }
            }
        },
        Tree::Branch(map) => {
            let types = map.iter().map(|(key, value)| {
                let key_ident = syn::Ident::new(key, proc_macro2::Span::call_site());

                let type_path = match value {
                    Tree::Branch(_) => format!("{}{}", path, key.to_pascal_case()),
                    Tree::Leaf(file) => file._type(),
                };

                let type_ident = syn::Ident::new(&type_path, proc_macro2::Span::call_site());

                quote! {
                    #key_ident: #type_ident
                }
            });

            let struct_declaration = quote! {
                struct #path_ident {
                    #(#types),*
                }
            };

            *top = quote! {
                #top
                #struct_declaration
            };

            let fields = map.into_iter().map(|(key, value)| {
                let key_ident = syn::Ident::new(&key, proc_macro2::Span::call_site());
                let nested_path = format!("{}{}", path, key.to_pascal_case());
                let value = gen_code(value, top, nested_path);

                quote! {
                    #key_ident: #value
                }
            });

            quote! {
                #path_ident {
                    #(#fields),*
                }
            }
        }
    }
}

// fn not_hidden(entry: &DirEntry) -> bool {
//     !entry
//         .file_name()
//         .to_str()
//         .map(|s| s.starts_with("."))
//         .unwrap_or(false)
// }

// fn process_tree(tree: Tree) -> Tree {
//     match tree {
//         Tree::Branch(map) => {
//             let mut new_branch = Tree::Branch(HashMap::new());
//
//             for (k, v) in map {
//                 let tree = match v {
//                     Tree::Leaf(file) => {
//                         let parts: Vec<_> = k
//                             .split('.')
//                             .filter(|s| !s.is_empty())
//                             .map(|e| e.to_string())
//                             .collect();
//
//                         parts_to_map(parts, file)
//                     }
//                     Tree::Branch(_) => process_tree(v),
//                 };
//                 dbg!(&tree);
//                 add_branch(&mut new_branch, &tree, &k);
//             }
//
//             new_branch
//         }
//         Tree::Leaf(file) => Tree::Leaf(file),
//     }
// }
//
// fn parts_to_map(mut parts: Vec<String>, end_file: File) -> Tree {
//     if parts.is_empty() {
//         return Tree::Leaf(end_file);
//     }
//
//     let part = parts.remove(0);
//
//     Tree::Branch(HashMap::from([(part, parts_to_map(parts, end_file))]))
// }
//
// fn add_branch(mother: &mut Tree, daughter: &Tree, branch_name: &str) {
//     match mother {
//         Tree::Branch(map) => {
//             if map.contains_key(branch_name) {
//                 match map.get_mut(branch_name).unwrap() {
//                     Tree::Leaf(_) => panic!("Folder and file with same name."),
//                     inner_branch @ Tree::Branch(_) => match daughter {
//                         Tree::Leaf(_) => panic!("Folder and file with same name."),
//                         Tree::Branch(other_map) => {
//                             for key in other_map.keys() {
//                                 let value = other_map.get(key).unwrap();
//                                 add_branch(inner_branch, value, key)
//                             }
//                         }
//                     },
//                 }
//             }
//         }
//         _ => unreachable!(),
//     }
// }
