// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use lsp_types::{Command, Location, Position};
use move_command_line_common::files::FileHash;
use move_ir_types::location::*;
use move_package::source_package::layout::SourcePackageLayout;

use std::collections::HashMap;
use std::{path::*, vec};

/// Double way mapping between FileHash and FilePath.
#[derive(Debug, Default)]
pub struct PathBufHashMap {
    path_2_hash: HashMap<PathBuf, FileHash>,
    hash_2_path: HashMap<FileHash, PathBuf>,
}

impl PathBufHashMap {
    pub fn update(&mut self, path: PathBuf, hash: FileHash) {
        if let Some(hash) = self.path_2_hash.get(&path) {
            self.hash_2_path.remove(&hash);
        }
        self.path_2_hash.insert(path.clone(), hash.clone());
        self.hash_2_path.insert(hash, path);
    }
    pub(crate) fn get_hash(&self, path: &PathBuf) -> Option<&'_ FileHash> {
        self.path_2_hash.get(path)
    }
    pub(crate) fn get_path(&self, hash: &FileHash) -> Option<&'_ PathBuf> {
        self.hash_2_path.get(hash)
    }
}
/// A thin wrapper on `FileLineMapping`
/// Sometimes only handle one file.
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct FileLineMappingOneFile {
    mapping: FileLineMapping,
}

impl From<FileLineMapping> for FileLineMappingOneFile {
    fn from(value: FileLineMapping) -> Self {
        Self { mapping: value }
    }
}

impl FileLineMappingOneFile {
    pub fn update(&mut self, content: &str) {
        self.mapping.update(Path::new(".").to_path_buf(), content);
    }
    pub(crate) fn translate(
        &self,
        start_index: ByteIndex,
        end_index: ByteIndex,
    ) -> Option<lsp_types::Range> {
        self.mapping
            .translate(&Path::new(".").to_path_buf(), start_index, end_index)
            .map(|x| x.mk_range())
    }
}

#[derive(Debug, Default)]
pub struct FileLineMapping {
    m: HashMap<PathBuf /* filepath */, Vec<ByteIndex>>,
}

impl FileLineMapping {
    pub fn update(&mut self, filepath: PathBuf, content: &str) {
        let mut v = vec![0];
        for (index, s) in content.as_bytes().iter().enumerate() {
            // TODO how to support windows \r\n
            if *s == 10 {
                // \n
                v.push((index + 1) as ByteIndex);
            }
        }
        if let Some(last) = content.as_bytes().last() {
            if *last != 10 {
                v.push((content.as_bytes().len()) as ByteIndex);
            }
        }
        self.m.insert(filepath, v);
    }

    pub fn translate(
        &self,
        filepath: &PathBuf,
        start_index: ByteIndex,
        mut end_index: ByteIndex,
    ) -> Option<FileRange> {
        if end_index < start_index {
            // maybe something goes wrong with syntax.rs
            // sometimes end_index < start_index.
            // this is a dummy fix.
            end_index = start_index;
        }
        let vec = self.m.get(filepath)?;
        let too_big = vec.last().map(|x| *x <= end_index).unwrap_or(false);
        if too_big {
            return None;
        }
        fn search(vec: &[ByteIndex], byte_index: ByteIndex) -> (u32, u32) {
            let mut index = bisection::bisect_left(vec, &byte_index);
            if vec[index] != byte_index {
                index = index - 1;
            }
            (index as u32, byte_index - vec[index as usize])
        }

        let (line_start, col_start) = search(&vec[..], start_index);
        let end = if let Some(t) = vec.get(line_start as usize + 1) {
            if *t > end_index {
                // Most case O(1) so we can have the same result but more fast.
                Some((line_start, end_index - vec[line_start as usize]))
            } else {
                None
            }
        } else {
            None
        };
        let (line_end, col_end) = end.unwrap_or(search(&vec[..], end_index));
        Some(FileRange {
            path: filepath.clone(),
            line_start,
            col_start,
            line_end,
            col_end,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn file_mapping() {
        let filepath = PathBuf::from("test");

        let mut f = FileLineMapping::default();
        f.update(
            filepath.clone(),
            r#"123456
123456
abc        "#,
        );

        let r = f.translate(&filepath, 0, 2).unwrap();
        assert_eq!(
            r,
            FileRange {
                path: filepath.clone(),
                line_start: 0,
                line_end: 0,
                col_start: 0,
                col_end: 2
            }
        );

        let r = f.translate(&filepath, 9, 10).unwrap();
        assert_eq!(
            r,
            FileRange {
                path: filepath.clone(),
                line_start: 1,
                line_end: 1,
                col_start: 2,
                col_end: 3
            }
        );
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileRange {
    pub path: PathBuf,
    /// Start.
    pub line_start: u32,
    pub col_start: u32,

    /// End.
    pub line_end: u32,
    pub col_end: u32,
}

impl FileRange {
    pub fn mk_location(&self) -> lsp_types::Location {
        let range = self.mk_range();
        let uri = url::Url::from_file_path(self.path.as_path()).unwrap();
        lsp_types::Location::new(uri, range)
    }
    pub fn mk_range(&self) -> lsp_types::Range {
        lsp_types::Range {
            start: lsp_types::Position {
                line: self.line_start,
                character: self.col_start,
            },
            end: Position {
                line: self.line_end,
                character: self.col_end,
            },
        }
    }
}

impl std::fmt::Display for FileRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}:({},{}):({},{})",
            self.path.as_path(),
            self.line_start,
            self.col_start,
            self.line_end,
            self.col_end
        )
    }
}
impl FileRange {
    pub(crate) fn unknown() -> Self {
        Self {
            path: PathBuf::from("<unknown>"),
            line_start: 0,
            col_start: 0,
            col_end: 0,
            line_end: 0,
        }
    }
}

/// Path concat from
pub fn path_concat(p1: &Path, p2: &Path) -> PathBuf {
    let p2: Vec<_> = p2.components().collect();
    let is_abs = match p2.get(0).unwrap() {
        Component::RootDir | Component::Prefix(_) => true,
        _ => false,
    };
    let mut p1: Vec<_> = p1.components().collect();
    normal_path_components(if is_abs {
        &p2
    } else {
        {
            p1.extend(p2);
            &p1
        }
    })
}

/// concat Move.toml file.
pub fn path_concat_move_toml(p1: &Path, p2: &Path) -> PathBuf {
    let p1_is_move_toml = match p1.to_str() {
        Some(x) => x.ends_with(PROJECT_FILE_NAME),
        None => false,
    };
    if p1_is_move_toml {
        let mut p1 = p1.to_path_buf();
        p1.pop();
        path_concat(p1.as_path(), p2)
    } else {
        path_concat(p1, p2)
    }
}

pub fn normal_path_components<'a>(x: &Vec<Component<'a>>) -> PathBuf {
    let mut ret = PathBuf::new();
    for v in x {
        match v {
            Component::Prefix(x) => ret.push(x.as_os_str()),
            Component::RootDir => ret.push("/"),
            Component::CurDir => {}
            Component::ParentDir => {
                let _ = ret.pop();
            }
            Component::Normal(x) => ret.push(*x),
        }
    }
    if ret.to_str().unwrap() == "" {
        ret.push(".")
    }
    ret
}

pub(crate) fn normal_path(p: &Path) -> PathBuf {
    let x: Vec<_> = p.components().collect();
    normal_path_components(&x)
}

pub trait GetPosition {
    fn get_position(&self) -> (PathBuf, u32 /* line */, u32 /* col */);
    fn in_range(x: &impl GetPosition, range: &FileRange) -> bool {
        let (filepath, line, col) = x.get_position();
        if filepath != range.path.clone() {
            return false;
        }
        if line < range.line_start {
            return false;
        }
        if line == range.line_start && col < range.col_start {
            return false;
        }
        if line > range.line_end {
            return false;
        }
        if line == range.line_end && col > range.col_end {
            return false;
        }
        true
    }
}

pub struct GetPositionStruct {
    pub fpath: PathBuf,
    pub line: u32,
    pub col: u32,
}

impl GetPosition for GetPositionStruct {
    fn get_position(&self) -> (PathBuf, u32 /* line */, u32 /* col */) {
        (self.fpath.clone(), self.line, self.col)
    }
}

pub fn discover_manifest_and_kind(x: &Path) -> Option<(PathBuf, SourcePackageLayout)> {
    let mut x: Vec<_> = x.components().collect();
    // We should be able at least pop one.
    x.pop()?;
    let mut layout = None;
    while x.len() > 0 {
        while x.len() > 0 {
            layout = x
                .last()
                .map(|x| match x.as_os_str().to_str().unwrap() {
                    "tests" => Some(SourcePackageLayout::Tests),
                    "sources" => Some(SourcePackageLayout::Sources),
                    "scripts" => Some(SourcePackageLayout::Scripts),
                    _ => return None,
                })
                .flatten();
            if layout.is_some() {
                break;
            }
            x.pop();
        }
        let layout = layout.clone()?;
        // Pop tests or sources ...
        x.pop()?;
        let mut manifest_dir = PathBuf::new();
        for x in x.iter() {
            manifest_dir.push(x);
        }
        // check if manifest exists.
        let mut manifest_file = manifest_dir.clone();
        manifest_file.push(PROJECT_FILE_NAME);
        if manifest_file.exists() {
            return Some((manifest_dir, layout));
        }
    }
    None
}

#[test]
fn discover_manifest_and_kind_test() {
    let (_, kind) = discover_manifest_and_kind(
        PathBuf::from("/Users/yuyang/projects/test-move2/scripts/aaa.move").as_path(),
    )
    .unwrap();
    assert!(kind == SourcePackageLayout::Scripts);
    let (_, kind) = discover_manifest_and_kind(
        PathBuf::from("/Users/yuyang/projects/test-move2/sources/some.move").as_path(),
    )
    .unwrap();
    assert!(kind == SourcePackageLayout::Sources);
    let (_, kind) = discover_manifest_and_kind(
        PathBuf::from("/Users/yuyang/projects/test-move2/sources/configs/some.move").as_path(),
    )
    .unwrap();
    assert!(kind == SourcePackageLayout::Sources);
    let (_, kind) = discover_manifest_and_kind(
        PathBuf::from("/Users/yuyang/projects/test-move2/sources/tests/some.move").as_path(),
    )
    .unwrap();
    assert!(kind == SourcePackageLayout::Sources);
    let (_, kind) = discover_manifest_and_kind(
        PathBuf::from("/Users/yuyang/projects/test-move2/tests/some.move").as_path(),
    )
    .unwrap();
    assert!(kind == SourcePackageLayout::Tests);
}

pub fn is_sub_dir(p: PathBuf, mut sub: PathBuf) -> bool {
    while sub.pop() {
        if p == sub {
            return true;
        }
    }
    false
}

/// There command should implemented in `LSP` client.
pub enum MoveAnalyzerClientCommands {
    GotoDefinition(Location),
}

use lsp_types::Range;

#[derive(Clone, serde::Serialize)]
pub struct PathAndRange {
    range: Range,
    fpath: String,
}

impl From<&Location> for PathAndRange {
    fn from(value: &Location) -> Self {
        Self {
            range: value.range,
            fpath: value
                .uri
                .to_file_path()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        }
    }
}

pub const PROJECT_FILE_NAME: &str = "Move.toml";

#[cfg(not(target_os = "windows"))]
pub fn cpu_pprof(_seconds: u64) {
    use std::fs::File;
    use std::str::FromStr;
    use std::time::Duration;
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .blocklist(&["libc", "libgcc", "pthread", "vdso"])
        .build()
        .unwrap();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::new(_seconds, 0));
        match guard.report().build() {
            Result::Ok(report) => {
                // let mut tmp = std::env::temp_dir();
                let mut tmp = PathBuf::from_str("/Users/yuyang/.movefmt").unwrap();

                tmp.push("movefmt-flamegraph.svg");
                let file = File::create(tmp.clone()).unwrap();
                report.flamegraph(file).unwrap();
                eprintln!("pprof file at {:?}", tmp.as_path());
            }
            Result::Err(e) => {
                log::error!("build report failed,err:{}", e);
            }
        };
    });
}
#[cfg(target_os = "windows")]
pub fn cpu_pprof(_seconds: u64) {
    log::error!("Can't run pprof in Windows");
}
