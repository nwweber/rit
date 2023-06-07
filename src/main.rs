use std::fs;
use std::path;
use clap::{Parser, Subcommand, ValueEnum};

/// rit - git, but in rust, and definitely not complete
#[derive(Debug, Parser)] // requires `derive` feature
#[command(long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Initializes a new, empty Git repository. Fails if ".git" folder already exists in worktree_root.
    Init {
        /// Path to the folder that should become the new worktree root, i.e. in which we will create ".git".
        worktree_root: String,
    },
    /// Computes git object representation of given object_type.
    HashObject {
        /// What kind of Git object do we wish to make?
        object_type: GitObjectType,
        /// Path to the file we wish to hash.
        filepath: path::PathBuf,
        /// Actually write generated object to Git object store of current repo.
        do_write: Option<bool>,
    }
}


fn main() {
    let args2 = Cli::parse();

    match args2.command {
        Commands::HashObject {
            do_write,
            object_type,
            filepath,
        } => {
            cmd_hash_object(do_write, object_type, &filepath);
        }
        Commands::Init {worktree_root}  => {
            cmd_init(&worktree_root).unwrap();
        }
    }


    // let args: Vec<String> = env::args().collect();
    // let command = &args[1];
    // if command == "init" {
            // cmd_init("testing");
    // } else if command == "hash-object" {
            // cmd_hash_object(Some(false), GitObjectType::Blob, path::Path::new("hello.txt"));
    // } else {
            // print!("something else");
    // }
}

/// 'init' generates the files necessary for an empty git repository
fn cmd_init(worktree_root: &str) -> Result<(), &str> {
    println!("init :)");
    println!("root dir: {}", worktree_root);
    let git_root: path::PathBuf = [worktree_root,  ".git"].iter().collect();
    if git_root.exists() {
        return Err("git root already exists")
    }
    fs::create_dir_all(&git_root).unwrap();
    fs::create_dir_all(git_root.join("objects")).unwrap();
    let refs_root = git_root.join("refs");
    fs::create_dir_all(refs_root.join("heads")).unwrap();
    fs::create_dir_all(refs_root.join("tags")).unwrap();
    fs::write(git_root.join("description"), "ce n'est pas un dépôt git").unwrap();
    fs::write(git_root.join("HEAD"), "ref: refs/heads/main\n").unwrap();

    // weird line endings to get a) newlines but also b) skip following whitespace indent
    let default_config = "\
    [core]\n\
    repositoryformatversion = 0\n\
    filemode = false\n\
    bare = false\n";
    fs::write(git_root.join("config"), default_config).unwrap();

    return Ok(())
}


/// The different kinds of objects Git knows about.
#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum GitObjectType {
    /// Just a bunch of data.
    Blob,
    /// Represents a file system.
    Tree,
    /// A link to another object.
    Tag,
    /// A Git commit.
    Commit,
}

/// create GitObject of given type, using data located at filepath; write to git object storage if do_write is true
fn cmd_hash_object(do_write: Option<bool>, object_type: GitObjectType, filepath: &path::Path) {
   println!("doing hash object"); 
   // get file contents
   let mut file_contents = fs::read(filepath).expect("expected to be able to read file for hashing");
   //dbg!(file_contents);
   // construct git obj contents
   let git_type = match object_type {
        GitObjectType::Blob => "blob",
        GitObjectType::Tree => "tree",
        GitObjectType::Tag => "tag",
        GitObjectType::Commit => "commit",
   };
   let contents_length = file_contents.len();
   // length of file contents plus a bit - can count it out exactly later
   let mut object_bytes: Vec<u8> = Vec::with_capacity(contents_length + 50);
   // format of git object:
   // object_type0x20size_in_bytes0x00contents
   for byte in git_type.bytes(){
        object_bytes.push(byte);
    }
   let ascii_whitespace = 32;
   object_bytes.push(ascii_whitespace);
   // converting to string so we can iterate over the individual digits
   for c in contents_length.to_string().bytes() {
           object_bytes.push(c);
   }
   object_bytes.push(0);
   object_bytes.append(&mut file_contents);
   dbg!(&object_bytes);
   //dbg!(String::from(contents_length.to_string()));
   // hash
   // determine file path, including path to object dir
   // if write: write using zlib compression

}
