extern crate git2;

use git2::*;
use std::path::Path;
use std::path::PathBuf;

pub struct GitPromptRepo {
    lg2_repo: Option<Repository>,
    origin_path: PathBuf,
    root_path: PathBuf,
    is_unborn: bool,
    has_head: bool,
}


impl GitPromptRepo {
    pub fn new(path_spec: &Path) -> GitPromptRepo {
        let lg2_repo = Repository::discover(path_spec).ok();
        let mut root_path = PathBuf::new();
        let mut is_unborn = false;
        let mut has_head = false;

        if let Some(ref repo) = lg2_repo {
            root_path = repo.path().to_path_buf();
            if let Err(e) = repo.head() {
                is_unborn = e.code() == git2::ErrorCode::UnbornBranch;
            } else {
                has_head = true;
            }
        }

        GitPromptRepo {
            lg2_repo: lg2_repo,
            origin_path: path_spec.to_path_buf(),
            root_path: root_path,
            is_unborn: is_unborn,
            has_head: has_head,
        }
    }

    // Assumes has_head is true
    fn head_name(&self) -> String {
        self.lg2_repo
            .as_ref()
            .unwrap()
            .head()
            .unwrap()
            .name()
            .unwrap_or("")
            .trim_left_matches("refs/heads/")
            .to_string()
    }

    fn head_to_branch(&self) -> Option<Branch> {
        if self.has_head {
            let head_name = self.head_name();
            return self.lg2_repo
                .as_ref()
                .unwrap()
                .find_branch(head_name.as_str(), BranchType::Local)
                .ok();
        }
        None
    }

    fn is_branch_with_upstream(&self) -> bool {
        if let Some(branch) = self.head_to_branch() {
            return branch.upstream().is_ok();
        }
        false
    }

    // fn is_remote(&self) -> bool {
    //     match &self.ref_result {
    //         &Ok(ref r) => r.is_remote(),
    //         &Err(_) => false
    //     }
    // }

    fn ref_name(&self) -> String {
        let mut ref_string = "".to_string();
        if self.is_unborn {
            ref_string = "[Unborn]".to_string();
        } else if self.has_head {
            ref_string = self.head_name();
        }
        ref_string.to_string()
    }

    fn upstream_name(&self) -> String {
        if self.is_branch_with_upstream() {
            let branch = self.head_to_branch().unwrap();
            let upstream_branch = branch.upstream().unwrap();
            if let Ok(Some(name)) = upstream_branch.name() {
                return name.to_string();
            }
        }
        "".to_string()
    }

    fn ahead_behind(&self) -> String {
        let mut result = String::from("");
        if self.is_branch_with_upstream() {
            result = String::from("↓·3↑·1");
        }
        result
    }
}


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
        prompt += "/";
        prompt += upstream_name_part.as_str();
    }
    println!("{}", prompt);
}
