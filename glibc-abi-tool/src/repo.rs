// Copyright 2022 Gregory Szorc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::abilist::{ABIList, ABILists, VersionedAbiLists};
use anyhow::{Context, Result, anyhow};
use gix::date::time::format::SHORT;
use gix::reference::Category;
use gix::{Repository, ThreadSafeRepository};
use gix_ref::Reference;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

/// Represents a glibc x.y[.z] version.
#[derive(Clone, Copy, Debug, Hash, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct GlibcVersion {
    pub major: u8,
    pub minor: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<u8>,
}

impl GlibcVersion {
    /// Format the X.Y version string.
    pub fn major_minor(self) -> String {
        format!("{}.{}", self.major, self.minor)
    }

    /// Format the X.Y[.Z] version string.
    pub fn major_minor_patch(self) -> String {
        if let Some(patch) = self.patch {
            format!("{}.{}.{}", self.major, self.minor, patch)
        } else {
            format!("{}.{}", self.major, self.minor)
        }
    }
}

/// A glibc Git tag.
#[derive(Debug, Eq, PartialEq)]
pub struct Tag {
    pub tag: String,
    pub semver_version: semver::Version,
    pub version: GlibcVersion,
    pub tag_id: gix::ObjectId,
    pub commit_id: gix::ObjectId,
    pub commit_date: String,
}

impl PartialOrd<Self> for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> Ordering {
        self.semver_version.cmp(&other.semver_version)
    }
}

impl Tag {
    pub fn from_reference(repo: &Repository, r: &Reference) -> Result<Option<Tag>> {
        if let Some((category, full_tag)) = r.name.category_and_short_name()
            && category == Category::Tag
        {
            if let Some(tag_s) = full_tag.to_string().strip_prefix("glibc-") {
                let tag_id = r.target.clone().into_id();
                let tag = repo.find_tag(tag_id)?;
                let commit_id = tag.target_id()?.detach();
                let semver_version = semver::Version::parse(tag_s)?;
                let glibc_version = GlibcVersion {
                    major: semver_version.major as _,
                    minor: semver_version.minor as _,
                    patch: None,
                };
                let commit = repo.find_commit(commit_id)?;
                let t = commit.time()?;
                let commit_date = t.format(SHORT)?;

                Ok(Some(Tag {
                    tag: full_tag.to_string(),
                    semver_version,
                    version: glibc_version,
                    tag_id,
                    commit_id,
                    commit_date,
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

/// Interfaces with the glibc Git repository.
pub struct Repo {
    repo: ThreadSafeRepository,
}

impl Repo {
    /// Create a new instance by opening the specified the Git.
    pub fn open(repo_path: impl Into<PathBuf>) -> Result<Self> {
        let repo = gix::open(repo_path)?.into_sync();

        Ok(Self { repo })
    }

    /// Convert to a usable repository instance.
    pub fn to_repo(&self) -> Repository {
        self.repo.to_thread_local()
    }

    /// Resolve glibc Git tags.
    ///
    /// Version is the parsed glibc version. Returned object ID should refer to the Git commit ID.
    pub fn tags(&self) -> Result<Vec<Tag>> {
        let repo = self.to_repo();
        Ok(repo
            .refs
            .iter()?
            .all()?
            .filter_map(|r| {
                if let Ok(r) = r {
                    if let Ok(Some(tag)) = Tag::from_reference(&repo, &r) {
                        Some(tag)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>())
    }

    /// Like tags but resolves the latest tag within an X.Y glibc release.
    pub fn latest_tags(&self) -> Result<Vec<Tag>> {
        let mut tags = self.tags()?;
        tags.sort();

        let mut seen_majmin = HashSet::new();
        let mut res = vec![];
        for tag in tags.into_iter().rev() {
            if seen_majmin.contains(&tag.version) {
                continue;
            }

            seen_majmin.insert(tag.version);
            res.push(tag);
        }

        Ok(res)
    }

    /// Resolves parsed ABI lists for a given Git commit ID.
    ///
    /// This finds .abilist files for libraries, parses them, then returns that full set
    /// as a data structure.
    pub fn library_abilists_for_commit(
        &self,
        commit_id: impl Into<gix::ObjectId>,
    ) -> Result<ABILists> {
        let repo = self.repo.to_thread_local();

        let commit = repo.find_commit(commit_id)?;
        let root_tree = commit.tree()?;

        // Look for the sysdeps directory.
        let sysdeps_entry = root_tree
            .find_entry("sysdeps")
            .ok_or_else(|| anyhow!("failed to find sysdeps/"))?;
        let sysdeps_tree = repo.find_tree(sysdeps_entry.oid())?;

        let files = sysdeps_tree.traverse().breadthfirst.files()?;

        let mut abilists = ABILists::default();

        for file in files {
            let file_path = PathBuf::from("sysdeps").join(file.filepath.to_string());

            let filename = file_path
                .file_name()
                .expect("should have file name")
                .to_string_lossy()
                .to_string();

            // Only care about .abilist files belonging to libraries.
            if !filename.ends_with(".abilist") || !filename.starts_with("lib") {
                continue;
            }

            let blob = repo.find_blob(file.oid)?;

            // Some files are empty. Don't waste time.
            if blob.data.is_empty() {
                continue;
            }
            let abilist =
                ABIList::parse(&blob.data).with_context(|| format!("parsing {}", file.filepath))?;

            abilists.insert(file_path, abilist);
        }

        Ok(abilists)
    }

    /// Resolve ABI lists for all glibc versions.
    pub fn library_versioned_abilists(&self) -> Result<VersionedAbiLists> {
        let m = self
            .latest_tags()?
            .into_par_iter()
            .filter(|tag| {
                // .abilist files introduced in 2.16. Don't waste time on older versions.
                tag.version.ge(&GlibcVersion {
                    major: 2,
                    minor: 16,
                    patch: None,
                })
            })
            .map(|tag| {
                let mut abilists = self.library_abilists_for_commit(tag.commit_id)?;

                for (_, list) in abilists.iter_mut() {
                    list.sort_common();
                }

                Ok((tag.version, abilists))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;

        Ok(m.into())
    }
}
