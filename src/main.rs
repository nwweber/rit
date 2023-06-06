use std::env;
use std::fs;
use std::path;
use clap::{Args, Parser, Subcommand, ValueEnum};

/// Rit :)
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "rit")]
#[command(about = "Rit :)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init {
        worktree_root: String,
    },
    HashObject {
        object_type: GitObjectType,
        filepath: path::PathBuf,
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
            cmd_init(&worktree_root);
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
    fs::create_dir_all(&git_root);
    fs::create_dir_all(git_root.join("objects"));
    let refs_root = git_root.join("refs");
    fs::create_dir_all(refs_root.join("heads"));
    fs::create_dir_all(refs_root.join("tags"));
    fs::write(git_root.join("description"), "ce n'est pas un dépôt git");
    fs::write(git_root.join("HEAD"), "ref: refs/heads/main\n");

    // weird line endings to get a) newlines but also b) skip following whitespace indent
    let default_config = "\
    [core]\n\
    repositoryformatversion = 0\n\
    filemode = false\n\
    bare = false\n";
    fs::write(git_root.join("config"), default_config);

    return Ok(())
}


#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum GitObjectType {
    Blob,
    Tree,
    Tag,
    Commit,
}

/// create GitObject of given type, using data located at filepath; write to git object storage if do_write is true
fn cmd_hash_object(do_write: Option<bool>, object_type: GitObjectType, filepath: &path::Path) -> Result<(), &str> {
   println!("doing hash object"); 
   // get file contents
   let file_contents = fs::read(filepath);
   //dbg!(file_contents);
   // construct git obj contents
   // hash
   // determine file path, including path to object dir
   // if write: write using zlib compression

   return Ok(())
}
