use std::fs;
use std::path;
use clap::{Parser, Subcommand, ValueEnum};
use sha1::{Sha1, Digest};
use hex;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::io::Write;

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
/// writes hash to stdout. objects in storage always zlib-compressed
fn cmd_hash_object(do_write: Option<bool>, object_type: GitObjectType, filepath: &path::Path) {
   let do_write = do_write.unwrap_or(false);
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
   object_bytes.extend(git_type.bytes());
   let ascii_whitespace = 0x20;
   object_bytes.push(ascii_whitespace);
   // converting to string so we can iterate over the individual digits
   // for length 1234 we need to add '1' '2' '3' '4' as ascii/bytes
   object_bytes.extend(contents_length.to_string().bytes());
   // ascii NULL-character is boundary between length and content
   let ascii_null_char = 0x00;
   object_bytes.push(ascii_null_char);
   object_bytes.append(&mut file_contents);
   // hash
   // let mut hasher = Sha1::new();
   let object_digest_hex = hex::encode(Sha1::digest(&object_bytes));
   println!("{}", object_digest_hex);
   // determine file path, including path to object dir

   /// looks for a ".git" folder contained in `search_path`. if found will return full path
   /// to that ".git" folder. if not, will recurse one directory up, all the way till root.
   fn find_git_root(search_path: path::PathBuf) -> Result<path::PathBuf, &'static str> {
           // we need to make an absolute path out of something like '.', otherwise `.parent()`
           // below will stop right away
           let search_path = search_path.canonicalize().unwrap();
           let candidate_root = search_path.join(".git");
           if candidate_root.is_dir() {
                   return Ok(candidate_root);
            } 
           // .git does not exist in search_path
           match search_path.parent() {
                   Some(parent) => {
                           find_git_root(path::PathBuf::from(parent))
                   },
                   None => Err("could not find git root"),
           }
   }

   let search_path = path::PathBuf::from(".");
   let git_root = find_git_root(search_path).expect("could not locate any git root");

   let mut object_path = git_root;
   object_path.push("objects");
   // first 2 digits of hash are folder
   object_path.push(&object_digest_hex[..2]);
   // rest is filename
   object_path.push(&object_digest_hex[2..]);
   // if write: write using zlib compression
   if do_write {
           let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
           encoder.write_all(object_bytes.as_slice()).unwrap();
           let compressed = encoder.finish().expect("could not compress object");
            fs::create_dir_all(object_path.parent().unwrap()).unwrap();
            fs::write(&object_path, &compressed).unwrap();
            println!("wrote {:?} bytes to {:?}", compressed.len(), object_path);
   }
}
