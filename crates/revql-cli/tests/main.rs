#![cfg(test)]

use std::path::Path;
use std::process::Command;
use std::{fs, io};
use tempfile::NamedTempFile;

fn get_test_file_path(name: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/")
        .join(name)
        .to_string_lossy()
        .to_string()
}

fn get_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_revql"))
}

#[test]
fn test_basic() -> io::Result<()> {
    let output_file = NamedTempFile::new()?;
    let path = get_test_file_path("sample.json");
    get_command()
        .args([path.as_str(), "Image"])
        .args(["--output-file", output_file.path().to_string_lossy().as_ref()])
        .output()?;

    let output = fs::read_to_string(output_file)?;
    assert!(output.contains("Image: Query.getUser -> User.profileImage"));
    assert!(output.contains("Image: Query.getUser -> User.blogs -> Blog.coverImage"));
    assert!(output.contains("Image: Query.getBlog -> Blog.coverImage"));
    assert!(output.contains("Image: Query.getBlog -> Blog.author -> User.profileImage"));
    Ok(())
}

#[test]
fn test_ignore_case() -> io::Result<()> {
    let output_file = NamedTempFile::new()?;
    let path = get_test_file_path("sample.json");
    get_command()
        .args([path.as_str(), "blog"])
        .args(["--output-file", output_file.path().to_string_lossy().as_ref()])
        .args(["--ignore-case"])
        .output()?;

    let output = fs::read_to_string(output_file)?;
    assert!(output.contains("Blog: Query.getUser -> User.blogs"));
    assert!(output.contains("Blog: Query.getBlog"));
    Ok(())
}

#[test]
fn test_contains() -> io::Result<()> {
    let output_file = NamedTempFile::new()?;
    let path = get_test_file_path("sample.json");
    get_command()
        .args([path.as_str(), "Us"])
        .args(["--output-file", output_file.path().to_string_lossy().as_ref()])
        .args(["--contains"])
        .output()?;

    let output = fs::read_to_string(output_file)?;
    assert!(output.contains("User: Query.getUser"));
    assert!(output.contains("User: Query.getBlog -> Blog.author"));
    Ok(())
}

#[test]
fn test_contains_ignore_case() -> io::Result<()> {
    let output_file = NamedTempFile::new()?;
    let path = get_test_file_path("sample.json");
    get_command()
        .args([path.as_str(), "us"])
        .args(["--output-file", output_file.path().to_string_lossy().as_ref()])
        .args(["--contains", "--ignore-case"])
        .output()?;

    let output = fs::read_to_string(output_file)?;
    assert!(output.contains("User: Query.getUser"));
    assert!(output.contains("User: Query.getBlog -> Blog.author"));
    Ok(())
}

#[test]
fn test_search_mode_field() -> io::Result<()> {
    let output_file = NamedTempFile::new()?;
    let path = get_test_file_path("sample.json");
    get_command()
        .args([path.as_str(), "id"])
        .args(["--output-file", output_file.path().to_string_lossy().as_ref()])
        .args(["--search-mode", "field"])
        .output()?;

    let output = fs::read_to_string(output_file)?;
    assert!(output.contains("User: Query.getUser"));
    assert!(output.contains("User: Query.getBlog -> Blog.author"));
    assert!(output.contains("Image: Query.getUser -> User.profileImage"));
    assert!(output.contains("Image: Query.getUser -> User.blogs -> Blog.coverImage"));
    assert!(output.contains("Image: Query.getBlog -> Blog.coverImage"));
    assert!(output.contains("Image: Query.getBlog -> Blog.author -> User.profileImage"));
    assert!(output.contains("Blog: Query.getUser -> User.blogs"));
    assert!(output.contains("Blog: Query.getBlog"));
    Ok(())
}

#[test]
fn test_search_mode_type_description() -> io::Result<()> {
    let output_file = NamedTempFile::new()?;
    let path = get_test_file_path("sample.json");
    get_command()
        .args([path.as_str(), "a user"])
        .args(["--output-file", output_file.path().to_string_lossy().as_ref()])
        .args(["--contains", "--search-mode", "description"])
        .output()?;

    let output = fs::read_to_string(output_file)?;
    assert!(output.contains("User: Query.getUser"));
    assert!(output.contains("User: Query.getBlog -> Blog.author"));
    Ok(())
}

#[test]
fn test_search_mode_field_description() -> io::Result<()> {
    let output_file = NamedTempFile::new()?;
    let path = get_test_file_path("sample.json");
    get_command()
        .args([path.as_str(), "id of the"])
        .args(["--output-file", output_file.path().to_string_lossy().as_ref()])
        .args(["--ignore-case", "--contains", "--search-mode", "description"])
        .output()?;

    let output = fs::read_to_string(output_file)?;
    assert!(output.contains("User: Query.getUser"));
    assert!(output.contains("User: Query.getBlog -> Blog.author"));
    Ok(())
}

#[test]
fn test_post_url() -> io::Result<()> {
    let output_file = NamedTempFile::new()?;
    get_command()
        .args(["--url", "https://graphqlzero.almansi.me/api", "User"])
        .args(["--output-file", output_file.path().to_string_lossy().as_ref()])
        .output()?;

    let output = fs::read_to_string(output_file)?;
    assert!(output.contains("User: Query.albums -> AlbumsPage.data -> Album.user"));
    assert!(output.contains("User: Query.album -> Album.user"));
    assert!(output.contains("User: Query.comments -> CommentsPage.data -> Comment.post -> Post.user"));
    assert!(output.contains("User: Query.comment -> Comment.post -> Post.user"));
    assert!(output.contains("User: Mutation.createAlbum -> Album.user"));
    assert!(output.contains("User: Mutation.updateAlbum -> Album.user"));
    assert!(output.contains("User: Mutation.createComment -> Comment.post -> Post.user"));
    assert!(output.contains("User: Mutation.updateComment -> Comment.post -> Post.user"));
    assert!(output.contains("User: Mutation.createUser"));
    assert!(output.contains("User: Mutation.updateUser"));
    Ok(())
}

#[test]
fn test_get_url() -> io::Result<()> {
    let output_file = NamedTempFile::new()?;
    get_command()
        .args(["--url", "https://graphqlzero.almansi.me/api", "User"])
        .args(["--output-file", output_file.path().to_string_lossy().as_ref()])
        .args(["--request", "GET"])
        .args(["--header", "Content-Type: application/json"])
        .args(["--header", "apollo-require-preflight: true"])
        .output()?;

    let output = fs::read_to_string(output_file)?;
    assert!(output.contains("User: Query.albums -> AlbumsPage.data -> Album.user"));
    assert!(output.contains("User: Query.album -> Album.user"));
    assert!(output.contains("User: Query.comments -> CommentsPage.data -> Comment.post -> Post.user"));
    assert!(output.contains("User: Query.comment -> Comment.post -> Post.user"));
    assert!(output.contains("User: Mutation.createAlbum -> Album.user"));
    assert!(output.contains("User: Mutation.updateAlbum -> Album.user"));
    assert!(output.contains("User: Mutation.createComment -> Comment.post -> Post.user"));
    assert!(output.contains("User: Mutation.updateComment -> Comment.post -> Post.user"));
    assert!(output.contains("User: Mutation.createUser"));
    assert!(output.contains("User: Mutation.updateUser"));
    Ok(())
}
