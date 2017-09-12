extern crate git_prompt;

use git_prompt::GitPromptRepo;
use std::path::Path;


fn main() {
    let repo = GitPromptRepo::new(Path::new("."));
    let mut prompt = "".to_string();

    let ref_name_part = repo.ref_name();
    let ahead_behind_part = repo.ahead_behind();
    let upstream_name_part = repo.upstream_name();

    if !ref_name_part.is_empty() {
        prompt += "[";
        prompt += ref_name_part.as_str();
        if !ahead_behind_part.is_empty() {
            prompt += " ";
            prompt += ahead_behind_part.as_str();
        }
        prompt += "]";
    }

    if !upstream_name_part.is_empty() {
        prompt += ":";
        prompt += upstream_name_part.as_str();
    }
    println!("{}", prompt);
}
