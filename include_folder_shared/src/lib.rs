#[derive(Debug, Clone)]
pub struct File {
    pub path: String,
    pub data: FileData,
}

#[derive(Debug, Clone)]
pub enum FileData {
    Blob(Vec<u8>),
    Text(String),
}

impl FileData {
    pub fn _type(&self) -> String {
        match self {
            Self::Blob(_) => "Vec<u8>",
            Self::Text(_) => "String",
        }
        .to_string()
    }
}

/// All generated structs implement this.
pub trait Directory {
    /// Gives you all the files in a directory or other such struct recursively.
    ///
    /// The paths returned from this will use periods as seperators as it looks when you are
    /// accessing the files as fields on a struct.
    ///
    /// A generated implementation may look like this:
    ///
    /// ```
    /// impl ::include_folder::Directory for TestDirSrc {
    ///     fn files(&self) -> Vec<::include_folder::File> {
    ///         use ::include_folder::Data;
    ///         <[_]>::into_vec(
    ///             #[rustc_box]
    ///             ::alloc::boxed::Box::new([
    ///                 ::include_folder::File {
    ///                     path: "nested.folders.test.txt".to_string(),
    ///                     data: self.nested.folders.test.txt.clone().to_file_data(),
    ///                 },
    ///                 ::include_folder::File {
    ///                     path: "parsing.lexer.txt".to_string(),
    ///                     data: self.parsing.lexer.txt.clone().to_file_data(),
    ///                 },
    ///                 ::include_folder::File {
    ///                     path: "parsing.lexer.rs".to_string(),
    ///                     data: self.parsing.lexer.rs.clone().to_file_data(),
    ///                 },
    ///                 ::include_folder::File {
    ///                     path: "main.rs".to_string(),
    ///                     data: self.main.rs.clone().to_file_data(),
    ///                 },
    ///             ]),
    ///         )
    ///     }
    /// }
    /// ```
    fn files(&self) -> Vec<File>;
}

pub trait Data {
    fn to_file_data(self) -> FileData;
}

impl Data for String {
    fn to_file_data(self) -> FileData {
        FileData::Text(self)
    }
}

impl Data for Vec<u8> {
    fn to_file_data(self) -> FileData {
        FileData::Blob(self)
    }
}
