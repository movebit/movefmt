use super::completion;
use super::goto_definition;
use super::project::*;
use crate::context::MultiProject;
use crate::utils::cpu_pprof;
use crate::utils::path_concat;
use log::{Level, Metadata, Record};

use std::path::PathBuf;
use std::str::FromStr;
struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            eprintln!("{} - {}", record.level(), record.args());
        }
    }
    fn flush(&self) {}
}

fn report_err(msg: String) {
    log::error!("{}", msg);
}
const LOGGER: SimpleLogger = SimpleLogger;

pub fn init_log() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap()
}

fn concat_current_working_dir(s: &str) -> PathBuf {
    path_concat(
        std::env::current_dir().unwrap().as_path(),
        PathBuf::from(s).as_path(),
    )
}

#[test]
fn goto_definition_test3() {
    init_log();
    let mut d = MultiProject::default();
    let m = Project::new(
        concat_current_working_dir("./tests/goto_definition"),
        &mut d,
        report_err,
    )
    .unwrap();
    let mut v = goto_definition::Handler::new(
        concat_current_working_dir("./tests/goto_definition/sources/test.move"),
        1,
        21,
    );
    m.run_full_visitor(&mut v);
    eprintln!("{:?}", v.result.unwrap());
}

#[test]
fn goto_definition_test2() {
    init_log();
    let mut d = MultiProject::default();
    let m = Project::new(
        concat_current_working_dir("/home/yuyang/projects/test-move"),
        &mut d,
        report_err,
    )
    .unwrap();
    let mut v = goto_definition::Handler::new(
        concat_current_working_dir("/home/yuyang/projects/test-move/sources/Hello.move"),
        119,
        21,
    );

    m.run_full_visitor(&mut v);
    eprintln!("{:?}", v.result.unwrap());
}

#[test]
fn completion() {
    init_log();
    let mut d = MultiProject::default();
    let m = Project::new("/Users/yuyang/projects/test-move", &mut d, report_err).unwrap();
    let mut v =
        completion::Handler::new("/Users/yuyang/projects/test-move/sources/some.move", 3, 28);
    m.run_full_visitor(&mut v);
    for x in v.result.unwrap().iter() {
        eprintln!("completion items:{:?} {:?} ", x.label, x.kind)
    }
}

#[test]
fn completion3() {
    init_log();
    let mut d = MultiProject::default();
    let m = Project::new(
        "/Users/yuyang/projects/aptos-core/aptos-move/framework/aptos-framework",
        &mut d,
        report_err,
    )
    .unwrap();
    let mut v =
        completion::Handler::new("/Users/yuyang/projects/aptos-core/aptos-move/framework/aptos-framework/sources/account.spec.move", 68, 50);
    m.run_full_visitor(&mut v);
    for x in v.result.unwrap().iter() {
        eprintln!("completion items:{:?} {:?} ", x.label, x.kind)
    }
}

#[test]
fn goto_definition_test4() {
    init_log();
    let mut d = MultiProject::default();
    let m = Project::new("/Users/yuyang/projects/test-move", &mut d, report_err).unwrap();
    let mut v =
        goto_definition::Handler::new("/Users/yuyang/projects/test-move/sources/some.move", 4, 25);
    m.run_full_visitor(&mut v);
    eprintln!("{:?}", v.result.unwrap());
}

#[test]
fn completion2() {
    init_log();
    let mut d = MultiProject::default();
    let m = Project::new("/Volumes/sanDisk/projects/test-move2", &mut d, report_err).unwrap();
    let mut v = completion::Handler::new(
        "/Volumes/sanDisk/projects/test-move2/sources/some.move",
        12,
        23,
    );
    m.run_full_visitor(&mut v);
    for x in v.result.unwrap().iter() {
        eprintln!("completion items:{:?} {:?} ", x.label, x.kind)
    }
}

#[test]
fn goto_definition_test() {
    init_log();
    let mut d = MultiProject::default();

    let m = Project::new(
        PathBuf::from_str("/Volumes/sanDisk/projects/sui/sui_programmability/examples/basics")
            .unwrap(),
        &mut d,
        report_err,
    )
    .unwrap();
    let mut v = goto_definition::Handler::new(
        concat_current_working_dir("./tests/goto_definition/sources/test.move"),
        1,
        21,
    );
    m.run_full_visitor(&mut v);
    eprintln!("{:?}", v.result.unwrap());
}

#[test]
fn goto_definition_test5() {
    init_log();
    cpu_pprof(10);
    init_log();
    let mut d = MultiProject::default();
    let _m = Project::new("/Volumes/sanDisk/projects/test-move", &mut d, report_err).unwrap();

    // let mut v = goto_definition::Handler::new(
    //     "/Volumes/sanDisk/projects/aptos-core/aptos-move/framework/aptos-stdlib/sources/simple_map.move",
    //     117,
    //     13,
    // );
    // m.run_full_visitor(&mut v);
    // eprintln!("{:?}", v.result.unwrap());
}
