extern crate rust_install;
extern crate tempdir;

use rust_install::{InstallPrefix, InstallType, NotifyHandler};
use rust_install::component::Transaction;
use rust_install::{temp, utils};
use rust_install::Error;
use tempdir::TempDir;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[test]
fn add_file() {
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    let mut file = tx.add_file("c", PathBuf::from("foo/bar")).unwrap();
    write!(&mut file, "test").unwrap();

    tx.commit();
    drop(file);

    assert_eq!(utils::raw::read_file(&prefix.path().join("foo/bar")).unwrap(),
               "test");
}

#[test]
fn add_file_then_rollback() {
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    tx.add_file("c", PathBuf::from("foo/bar")).unwrap();
    drop(tx);

    assert!(!utils::is_file(prefix.path().join("foo/bar")));
}

#[test]
fn add_file_that_exists() {
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    fs::create_dir_all(&prefixdir.path().join("foo")).unwrap();
    utils::write_file("", &prefixdir.path().join("foo/bar"), "").unwrap();

    let err = tx.add_file("c", PathBuf::from("foo/bar")).unwrap_err();

    match err {
        Error::ComponentConflict { name, path } => {
            assert_eq!(name, "c");
            assert_eq!(path, PathBuf::from("foo/bar"));
        }
        _ => panic!()
    }
}

#[test]
fn copy_file() {
    let srcdir = TempDir::new("multirust").unwrap();
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    let srcpath = srcdir.path().join("bar");
    utils::write_file("", &srcpath, "").unwrap();

    tx.copy_file("c", PathBuf::from("foo/bar"), &srcpath).unwrap();
    tx.commit();

    assert!(utils::is_file(prefix.path().join("foo/bar")));
}

#[test]
fn copy_file_then_rollback() {
    let srcdir = TempDir::new("multirust").unwrap();
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    let srcpath = srcdir.path().join("bar");
    utils::write_file("", &srcpath, "").unwrap();

    tx.copy_file("c", PathBuf::from("foo/bar"), &srcpath).unwrap();
    drop(tx);

    assert!(!utils::is_file(prefix.path().join("foo/bar")));
}

#[test]
fn copy_file_that_exists() {
    let srcdir = TempDir::new("multirust").unwrap();
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    let srcpath = srcdir.path().join("bar");
    utils::write_file("", &srcpath, "").unwrap();

    fs::create_dir_all(&prefixdir.path().join("foo")).unwrap();
    utils::write_file("", &prefixdir.path().join("foo/bar"), "").unwrap();

    let err = tx.copy_file("c", PathBuf::from("foo/bar"), &srcpath).unwrap_err();

    match err {
        Error::ComponentConflict { name, path } => {
            assert_eq!(name, "c");
            assert_eq!(path, PathBuf::from("foo/bar"));
        }
        _ => panic!()
    }
}

#[test]
fn copy_dir() {
    let srcdir = TempDir::new("multirust").unwrap();
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    let srcpath1 = srcdir.path().join("foo");
    let srcpath2 = srcdir.path().join("bar/baz");
    let srcpath3 = srcdir.path().join("bar/qux/tickle");
    utils::write_file("", &srcpath1, "").unwrap();
    fs::create_dir_all(srcpath2.parent().unwrap()).unwrap();
    utils::write_file("", &srcpath2, "").unwrap();
    fs::create_dir_all(srcpath3.parent().unwrap()).unwrap();
    utils::write_file("", &srcpath3, "").unwrap();

    tx.copy_dir("c", PathBuf::from("a"), srcdir.path()).unwrap();
    tx.commit();

    assert!(utils::is_file(prefix.path().join("a/foo")));
    assert!(utils::is_file(prefix.path().join("a/bar/baz")));
    assert!(utils::is_file(prefix.path().join("a/bar/qux/tickle")));
}

#[test]
fn copy_dir_then_rollback() {
    let srcdir = TempDir::new("multirust").unwrap();
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    let srcpath1 = srcdir.path().join("foo");
    let srcpath2 = srcdir.path().join("bar/baz");
    let srcpath3 = srcdir.path().join("bar/qux/tickle");
    utils::write_file("", &srcpath1, "").unwrap();
    fs::create_dir_all(srcpath2.parent().unwrap()).unwrap();
    utils::write_file("", &srcpath2, "").unwrap();
    fs::create_dir_all(srcpath3.parent().unwrap()).unwrap();
    utils::write_file("", &srcpath3, "").unwrap();

    tx.copy_dir("c", PathBuf::from("a"), srcdir.path()).unwrap();
    drop(tx);

    assert!(!utils::is_file(prefix.path().join("a/foo")));
    assert!(!utils::is_file(prefix.path().join("a/bar/baz")));
    assert!(!utils::is_file(prefix.path().join("a/bar/qux/tickle")));
}

#[test]
fn copy_dir_that_exists() {
    let srcdir = TempDir::new("multirust").unwrap();
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    fs::create_dir_all(prefix.path().join("a")).unwrap();

    let err = tx.copy_dir("c", PathBuf::from("a"), srcdir.path()).unwrap_err();

    match err {
        Error::ComponentConflict { name, path } => {
            assert_eq!(name, "c");
            assert_eq!(path, PathBuf::from("a"));
        }
        _ => panic!()
    }
}

#[test]
fn remove_file() {
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    let filepath = prefixdir.path().join("foo");
    utils::write_file("", &filepath, "").unwrap();

    tx.remove_file("c", PathBuf::from("foo")).unwrap();
    tx.commit();

    assert!(!utils::is_file(filepath));
}

#[test]
fn remove_file_then_rollback() {
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    let filepath = prefixdir.path().join("foo");
    utils::write_file("", &filepath, "").unwrap();

    tx.remove_file("c", PathBuf::from("foo")).unwrap();
    drop(tx);

    assert!(utils::is_file(filepath));
}

#[test]
fn remove_file_that_not_exists() {
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    let err = tx.remove_file("c", PathBuf::from("foo")).unwrap_err();

    match err {
        Error::ComponentMissingFile { name, path } => {
            assert_eq!(name, "c");
            assert_eq!(path, PathBuf::from("foo"));
        }
        _ => panic!()
    }
}

#[test]
fn remove_dir() {
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    let filepath = prefixdir.path().join("foo/bar");
    fs::create_dir_all(filepath.parent().unwrap()).unwrap();
    utils::write_file("", &filepath, "").unwrap();

    tx.remove_dir("c", PathBuf::from("foo")).unwrap();
    tx.commit();

    assert!(!utils::path_exists(filepath.parent().unwrap()));
}

#[test]
fn remove_dir_then_rollback() {
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    let filepath = prefixdir.path().join("foo/bar");
    fs::create_dir_all(filepath.parent().unwrap()).unwrap();
    utils::write_file("", &filepath, "").unwrap();

    tx.remove_dir("c", PathBuf::from("foo")).unwrap();
    drop(tx);

    assert!(utils::path_exists(filepath.parent().unwrap()));
}

#[test]
fn remove_dir_that_not_exists() {
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    let err = tx.remove_dir("c", PathBuf::from("foo")).unwrap_err();

    match err {
        Error::ComponentMissingDir { name, path } => {
            assert_eq!(name, "c");
            assert_eq!(path, PathBuf::from("foo"));
        }
        _ => panic!()
    }
}

#[test]
fn write_file() {
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    let content = "hi".to_string();
    tx.write_file("c", PathBuf::from("foo/bar"), content.clone()).unwrap();
    tx.commit();

    let path = prefix.path().join("foo/bar");
    assert!(utils::is_file(&path));
    let file_content = utils::raw::read_file(&path).unwrap();
    assert_eq!(content, file_content);
}

#[test]
fn write_file_then_rollback() {
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    let content = "hi".to_string();
    tx.write_file("c", PathBuf::from("foo/bar"), content.clone()).unwrap();
    drop(tx);

    assert!(!utils::is_file(&prefix.path().join("foo/bar")));
}

#[test]
fn write_file_that_exists() {
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    let content = "hi".to_string();
    utils::raw::write_file(&prefix.path().join("a"), &content).unwrap();
    let err = tx.write_file("c", PathBuf::from("a"), content.clone()).unwrap_err();

    match err {
        Error::ComponentConflict { name, path } => {
            assert_eq!(name, "c");
            assert_eq!(path, PathBuf::from("a"));
        }
        _ => panic!()
    }
}

// If the file does not exist, then the path to it is created,
// but the file is not.
#[test]
fn modify_file_that_not_exists() {
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    tx.modify_file(PathBuf::from("foo/bar")).unwrap();
    tx.commit();

    assert!(utils::path_exists(prefix.path().join("foo")));
    assert!(!utils::path_exists(prefix.path().join("foo/bar")));
}

// If the file does exist, then it's just backed up
#[test]
fn modify_file_that_exists() {
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    let ref path = prefix.path().join("foo");
    utils::raw::write_file(path, "wow").unwrap();
    tx.modify_file(PathBuf::from("foo")).unwrap();
    tx.commit();

    assert_eq!(utils::raw::read_file(path).unwrap(), "wow");
}

#[test]
fn modify_file_that_not_exists_then_rollback() {
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    tx.modify_file(PathBuf::from("foo/bar")).unwrap();
    drop(tx);

    assert!(!utils::path_exists(prefix.path().join("foo/bar")));
}

#[test]
fn modify_file_that_exists_then_rollback() {
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    let ref path = prefix.path().join("foo");
    utils::raw::write_file(path, "wow").unwrap();
    tx.modify_file(PathBuf::from("foo")).unwrap();
    utils::raw::write_file(path, "eww").unwrap();
    drop(tx);

    assert_eq!(utils::raw::read_file(path).unwrap(), "wow");
}

// This is testing that the backup scheme is smart enough not
// to overwrite the earliest backup.
#[test]
fn modify_twice_then_rollback() {
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    let ref path = prefix.path().join("foo");
    utils::raw::write_file(path, "wow").unwrap();
    tx.modify_file(PathBuf::from("foo")).unwrap();
    utils::raw::write_file(path, "eww").unwrap();
    tx.modify_file(PathBuf::from("foo")).unwrap();
    utils::raw::write_file(path, "ewww").unwrap();
    drop(tx);

    assert_eq!(utils::raw::read_file(path).unwrap(), "wow");
}

fn do_multiple_op_transaction(rollback: bool) {
    let srcdir = TempDir::new("multirust").unwrap();
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    // copy_file
    let relpath1 = PathBuf::from("bin/rustc");
    let relpath2 = PathBuf::from("bin/cargo");
    // copy_dir
    let relpath4 = PathBuf::from("doc/html/index.html");
    // modify_file
    let relpath5 = PathBuf::from("lib/rustlib/components");
    // write_file
    let relpath6 = PathBuf::from("lib/rustlib/rustc-manifest.in");
    // remove_file
    let relpath7 = PathBuf::from("bin/oldrustc");
    // remove_dir
    let relpath8 = PathBuf::from("olddoc/htm/index.html");

    let ref path1 = prefix.path().join(&relpath1);
    let ref path2 = prefix.path().join(&relpath2);
    let ref path4 = prefix.path().join(&relpath4);
    let ref path5 = prefix.path().join(&relpath5);
    let ref path6 = prefix.path().join(&relpath6);
    let ref path7 = prefix.path().join(&relpath7);
    let ref path8 = prefix.path().join(&relpath8);

    let ref srcpath1 = srcdir.path().join(&relpath1);
    fs::create_dir_all(srcpath1.parent().unwrap()).unwrap();
    utils::raw::write_file(srcpath1, "").unwrap();
    tx.copy_file("", relpath1, srcpath1).unwrap();

    let ref srcpath2 = srcdir.path().join(&relpath2);
    utils::raw::write_file(srcpath2, "").unwrap();
    tx.copy_file("", relpath2, srcpath2).unwrap();

    let ref srcpath4 = srcdir.path().join(&relpath4);
    fs::create_dir_all(srcpath4.parent().unwrap()).unwrap();
    utils::raw::write_file(srcpath4, "").unwrap();
    tx.copy_dir("", PathBuf::from("doc"), &srcdir.path().join("doc")).unwrap();

    tx.modify_file(relpath5).unwrap();
    utils::raw::write_file(path5, "").unwrap();

    tx.write_file("", relpath6, "".to_string()).unwrap();

    fs::create_dir_all(path7.parent().unwrap()).unwrap();
    utils::raw::write_file(path7, "").unwrap();
    tx.remove_file("", relpath7).unwrap();

    fs::create_dir_all(path8.parent().unwrap()).unwrap();
    utils::raw::write_file(path8, "").unwrap();
    tx.remove_dir("", PathBuf::from("olddoc")).unwrap();

    if !rollback {
        tx.commit();

        assert!(utils::path_exists(path1));
        assert!(utils::path_exists(path2));
        assert!(utils::path_exists(path4));
        assert!(utils::path_exists(path5));
        assert!(utils::path_exists(path6));
        assert!(!utils::path_exists(path7));
        assert!(!utils::path_exists(path8));
    } else {
        drop(tx);

        assert!(!utils::path_exists(path1));
        assert!(!utils::path_exists(path2));
        assert!(!utils::path_exists(path4));
        assert!(!utils::path_exists(path5));
        assert!(!utils::path_exists(path6));
        assert!(utils::path_exists(path7));
        assert!(utils::path_exists(path8));
    }
}

#[test]
fn multiple_op_transaction() {
    do_multiple_op_transaction(false);
}

#[test]
fn multiple_op_transaction_then_rollback() {
    do_multiple_op_transaction(true);
}

// Even if one step fails to rollback, rollback should
// continue to rollback other steps.
#[test]
fn rollback_failure_keeps_going() {
    let prefixdir = TempDir::new("multirust").unwrap();
    let txdir = TempDir::new("multirust").unwrap();

    let tmpnotify = temp::SharedNotifyHandler::none();
    let tmpcfg = temp::Cfg::new(txdir.path().to_owned(), tmpnotify);

    let prefix = InstallPrefix::from(prefixdir.path().to_owned(),
                                     InstallType::Owned);

    let notify = NotifyHandler::none();
    let mut tx = Transaction::new(prefix.clone(), &tmpcfg, notify);

    write!(&mut tx.add_file("", PathBuf::from("foo")).unwrap(), "").unwrap();
    write!(&mut tx.add_file("", PathBuf::from("bar")).unwrap(), "").unwrap();
    write!(&mut tx.add_file("", PathBuf::from("baz")).unwrap(), "").unwrap();

    fs::remove_file(prefix.path().join("bar")).unwrap();

    drop(tx);

    assert!(!utils::path_exists(prefix.path().join("foo")));
    assert!(!utils::path_exists(prefix.path().join("baz")));
}

// Test that when a transaction creates intermediate directories that
// they are deleted during rollback.
#[test]
#[ignore]
fn intermediate_dir_rollback() {
}
