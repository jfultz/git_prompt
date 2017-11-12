extern crate git_prompt;

use git_prompt::GitPromptRepo;
use std::path::Path;


fn main() {
    let repo = GitPromptRepo::new(Path::new("."));
    let mut prompt = "".to_string();

    let ref_name_part = repo.ref_name_head();
    let ahead_behind_part = repo.ahead_behind();
    let upstream_name_part = repo.upstream_name();
    let status_part = repo.status();

    if !ref_name_part.is_empty() {
        prompt += "[";
        prompt += ref_name_part.as_str();
        if !ahead_behind_part.is_empty() {
            prompt += " ";
            prompt += &ahead_behind_part;
        }
        if !status_part.is_empty() {
            prompt += "|";
            prompt += &status_part;
        }
        prompt += "]";
    }

    if !upstream_name_part.is_empty() {
        prompt += ":";
        prompt += &upstream_name_part;
    }
    println!("{}", prompt);
}
