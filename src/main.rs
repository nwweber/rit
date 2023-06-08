use clap::{Parser, Subcommand, ValueEnum};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use hex;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::Write;
use std::path;

/// rit - git, but in rust, and definitely not complete
#[derive(Debug, Parser)]
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
        #[arg(long, short, default_value="blob")]
        object_type: Option<GitObjectType>,
        /// Actually write generated object to Git object store of current repo.
        #[arg(short,long, required=false, num_args=0, default_value="false")]
        write: bool,
        /// Path to the file we wish to hash.
        //#[arg(long, short)]
        filepath: path::PathBuf,
    },
    /// Displays contents of file in object storage.
    CatFile {
            /// What kind of Git object we wish to cat.
            object_type: GitObjectType,
            /// The name of the object, currently only implemented for full sha1-digests
            object: String,
    },
}

fn main() {
    let args2 = Cli::parse();

    match args2.command {
        Commands::HashObject {
            object_type,
            write,
            filepath,
        } => {
            cmd_hash_object(write, object_type, &filepath);
        }
        Commands::Init { worktree_root } => {
            cmd_init(&worktree_root).unwrap();
        },
        Commands::CatFile {object_type, object } => {
            cmd_cat_file(object_type, object);
        },
    }
}

/// shows contents of `object`, assuming it is a `object_type`
fn cmd_cat_file(object_type: GitObjectType, object: String) {
    println!("cat-file coming soon :)");
}

/// 'init' generates the files necessary for an empty git repository
fn cmd_init(worktree_root: &str) -> Result<(), &str> {
    println!("init :)");
    println!("root dir: {}", worktree_root);
    let git_root: path::PathBuf = [worktree_root, ".git"].iter().collect();
    if git_root.exists() {
        return Err("git root already exists");
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

    return Ok(());
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

impl GitObjectType {
    fn name_as_string(&self) -> &str {
        // maybe i can have a more general solution in the future, e.g.
        // https://docs.rs/strum_macros/0.24.3/strum_macros/derive.Display.html
        // but nice to keep it basic for now :)
        match self {
            GitObjectType::Blob => "blob",
            GitObjectType::Tree => "tree",
            GitObjectType::Tag => "tag",
            GitObjectType::Commit => "commit",
        }
    }
}

/// create GitObject of given type, using data located at filepath; write to git object storage if do_write is true (default: false)
/// writes hash to stdout. objects in storage always zlib-compressed
fn cmd_hash_object(do_write: bool, object_type: Option<GitObjectType>, filepath: &path::Path) {
    let object_type = object_type.unwrap_or(GitObjectType::Blob);

    let git_type = object_type.name_as_string();
    let mut file_contents =
        fs::read(filepath).expect("expected to be able to read file for hashing");
    let contents_length = file_contents.len();
    // format of git object:
    // object_type0x20size_in_bytes0x00contents
    // length of file contents plus a bit - can count it out exactly later
    let mut object_bytes: Vec<u8> = Vec::with_capacity(contents_length + 50);
    object_bytes.extend(git_type.bytes());
    let ascii_whitespace = 0x20;
    object_bytes.push(ascii_whitespace);
    // converting to string so we can iterate over the individual digits
    // for length 1234 we need to add '1' '2' '3' '4' as ascii/bytes
    object_bytes.extend(contents_length.to_string().bytes());
    // ascii NULL-character is boundary between length and content
    let ascii_null_char = 0x00;
    object_bytes.push(ascii_null_char);
    // after this, all (non-zipped) git object content is done
    object_bytes.append(&mut file_contents);

    let object_digest_hex = hex::encode(Sha1::digest(&object_bytes));
    println!("{}", object_digest_hex);

    let git_root = find_git_root(None).expect("could not locate any git root");
    let mut object_path = git_root;
    object_path.push("objects");
    // first 2 digits of hash are folder
    object_path.push(&object_digest_hex[..2]);
    // rest is filename
    object_path.push(&object_digest_hex[2..]);

    if do_write {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(object_bytes.as_slice()).unwrap();
        let compressed = encoder.finish().expect("could not compress object");
        fs::create_dir_all(object_path.parent().unwrap()).unwrap();
        fs::write(&object_path, &compressed).unwrap();
        println!("wrote {:?} bytes to {:?}", compressed.len(), object_path);
    }
}

/// looks for a ".git" folder contained in `search_path` (default: "."). if found will return full path
/// to that ".git" folder. if not, will recurse one directory up, all the way till root.
fn find_git_root(search_path: Option<path::PathBuf>) -> Result<path::PathBuf, &'static str> {
    let search_path = search_path.unwrap_or(path::PathBuf::from("."));
    // we need to make an absolute path out of something like '.', otherwise `.parent()`
    // below will stop right away
    let search_path = search_path.canonicalize().unwrap();
    let candidate_root = search_path.join(".git");
    if candidate_root.is_dir() {
        return Ok(candidate_root);
    }
    // .git does not exist in search_path
    match search_path.parent() {
        Some(parent) => find_git_root(Some(parent.to_path_buf())),
        None => Err("could not find git root"),
    }
}
