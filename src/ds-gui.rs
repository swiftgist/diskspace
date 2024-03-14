use slint::{SharedString, VecModel};
use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};
use std::rc::Rc;

// slint::slint!(import { MainWindow, Rows } from "ds.slint";);

extern crate clap;
mod cli;
mod ds;
mod report;
mod report_gui;

use crate::ds::DSGroup;

fn main() {
    let matches = cli::get_matches();
    let anchors: Vec<_> = cli::get_dirs(&matches);
    let home = env::var("HOME").unwrap();
    let mut start = anchors.iter().nth(0).unwrap();
    if start == "./" {
        start = &home;
    }
    let last = Path::new(start).file_name().unwrap();

    // Initialize
    let mut group = DSGroup::new();
    let disk_space = group.calculate(&anchors, &matches);
    let list = report_gui::report_map(disk_space.sizes.clone(), &matches);

    // Slint
    let main_window = MainWindow::new().unwrap();
    let rows: Rc<VecModel<Rows>> = Default::default();
    let last_dir = base_dirname(last.to_string_lossy().to_string());
    populate_rows(list.clone(), disk_space.dirs.clone(), &rows);

    main_window.set_rows(rows.clone().into());
    main_window.set_last_dir(last_dir.clone());

    let ancestors: Rc<VecModel<SharedString>> = Default::default();
    let ancestor_paths: Rc<VecModel<SharedString>> = Default::default();

    populate_ancestors(
        SharedString::from(start),
        last_dir,
        &ancestors,
        &ancestor_paths,
    );

    main_window.set_ancestors(ancestors.clone().into());
    main_window.set_ancestor_paths(ancestor_paths.clone().into());

    let matches = matches.clone();
    let weak_window = main_window.as_weak();
    {
        main_window.on_changeDirectory(move |path| {
            // println!("Callback running...");
            let rows = rows.clone();
            let anchors = Vec::from([path.to_string()]);
            let mut group = DSGroup::new();
            let disk_space = group.calculate(&anchors, &matches);
            let list = report_gui::report_map(disk_space.sizes.clone(), &matches);

            let last_dir = base_dirname(path.to_string());

            let window = weak_window.unwrap();
            window.set_last_dir(last_dir.clone());

            clear_ancestors(&ancestors, &ancestor_paths);
            populate_ancestors(path, last_dir, &ancestors, &ancestor_paths);

            clear_rows(&rows);
            populate_rows(list.clone(), disk_space.dirs.clone(), &rows);
        });
    }
    main_window.run().unwrap();
}

/// clear_ancestors
///
/// Remove all elements from ancestors and ancestors_paths.  The clear() method is not
/// available.
fn clear_ancestors(
    ancestors: &Rc<VecModel<SharedString>>,
    ancestor_paths: &Rc<VecModel<SharedString>>,
) {
    ancestors.push(SharedString::from("...."));
    ancestor_paths.push(SharedString::from("...."));

    loop {
        let part = ancestors.remove(0);
        ancestor_paths.remove(0);
        if part == "...." {
            break;
        }
    }
}

/// populate_ancestors
///
/// Populate two Vecs.  One with individual directory names and the other will full
/// pathnames.  Do not include the current directory.
fn populate_ancestors(
    start: SharedString,
    last_dir: SharedString,
    ancestors: &Rc<VecModel<SharedString>>,
    ancestor_paths: &Rc<VecModel<SharedString>>,
) {
    let mut current_path = String::new();
    for dir in start.split('/').filter(|e| *e != "") {
        if last_dir != dir {
            ancestors.push(SharedString::from(dir));
            current_path.push_str(&format!("/{}", dir));
            ancestor_paths.push(SharedString::from(&current_path));
        }
    }
}

/// clear_rows
///
/// Remove all elements from rows.  The clear() method is not available.
fn clear_rows(rows: &Rc<VecModel<Rows>>) {
    // clear() and row_count() not working, add bogus entry to break loop
    rows.push(Rows {
        size: SharedString::from("0"),
        directory: false,
        path: SharedString::from("...."),
    });

    loop {
        let row = rows.remove(0);
        if row.path == "...." {
            break;
        }
    }
}

/// populate_rows
///
/// Populate a Vec of Rows.  Append filler entries to prevent gui resizing for now.
fn populate_rows(
    list: Vec<(String, String)>,
    dirs: BTreeMap<PathBuf, Vec<PathBuf>>,
    rows: &Rc<VecModel<Rows>>,
) {
    let end = &list.len();

    for (size, path) in list {
        rows.push(Rows {
            size: SharedString::from(size),
            directory: dirs.contains_key(&PathBuf::from(&path)),
            path: SharedString::from(path),
        });
    }

    // Append filler entries
    for _ in *end..20 {
        rows.push(Rows {
            size: SharedString::from(" "),
            directory: false,
            path: SharedString::from("..."),
        });
    }
}

/// base_dirname
///
/// Returns the last element of a directory path
fn base_dirname(current_dir: String) -> SharedString {
    let parts: Vec<_> = current_dir.split('/').filter(|e| *e != "").collect();

    let last_dir = match parts.last() {
        Some(last_dir) => SharedString::from(last_dir.to_string()),
        None => SharedString::from("/"),
    };

    last_dir
}

slint::slint! {
        import { ScrollView, GridBox, StandardTableView } from "std-widgets.slint";

        struct Rows { size: string, directory: bool, path: string }

        global Palette  {
            out property <color> window-background: #eee;
            out property <color> widget-background: #ddd;
            out property <color> widget-stroke: #888;
            out property <color> window-border: #ccc;
            out property <color> text-color: #666;
            out property <color> hyper-blue: #90d1ff;

        }

        component Label inherits Text {
            color: Palette.text-color;
        }

        component Button inherits TouchArea {
        in property <string> text <=> t.text;

            min-height: t.min-height;
            min-width: t.min-width + 10px;

            Rectangle {
                border-width: 3px;
                // border-color: root.has-hover ? Palette.widget-stroke : transparent;
                border-radius: 5px;
                // background: root.pressed ? Palette.widget-background.darker(30%) : Palette.widget-background;
                background: root.pressed ? Palette.widget-background.darker(30%) : #eee;

                t := Label {
                    font-weight: 800;
                    // font-size: 36px;
                    // y:0;
                    // color: #00a;
                    color: root.has-hover ? #0aa : #00a;
                    // width: 100%;
                    horizontal-alignment: center;
                }
            }
        }


        export component MainWindow inherits Window {
            in property <[Rows]> rows;
            in property <[string]> ancestors;
            in property <[string]> ancestor_paths;
            in property <string> parent_dir;
            in property <string> last_dir;
            // in property <[string]> subdirs;
            // in property <[string]> ancestors;

            callback changeDirectory(string);

            preferred-width: 1300px;
            preferred-height: 900px;
            min-width: 1330px;
            min-height: 870px;

            VerticalLayout {
                padding: 5px;
                spacing: 5px;

                Rectangle {
                    border-color: #f70;
                    border-width: 5px;
                    border-radius: 10px;
                    height: 6%;
                    width: 96%;

                    VerticalLayout {
                        padding: 3px;
                        spacing: 5px;

                        alignment: start;
                        HorizontalLayout {
                            spacing: 3px;
                            padding: 1px;
                            alignment: start;
                            Rectangle {
                                width: 5px;
                            }
                            Button {
                                clicked => { root.changeDirectory("/") }
                                text: "\u{1F5B4}";
                            }
                            Rectangle {
                                width: 50px;
                                Text {
                                    text: "/";
                                    vertical-alignment: center;
                                }
                            }

                            for dir[idx] in ancestors: HorizontalLayout {
                                padding: 1px;
                                spacing: 3px;
                                Rectangle {
                                    // width: 70px;
                                    Button {
                                        text: dir;
                                        clicked => { root.changeDirectory(ancestor_paths[idx]) }
                                    }
                                }
                                Rectangle {
                                    width: 20px;
                                    Text {
                                        text: "/";
                                        vertical-alignment: center;
                                    }
                                }
                            }
                            Text {
                                font-weight: 800;
                                text: last_dir;
                                vertical-alignment: center;
                            }
                        }
                   }
                }

                Rectangle {
                    border-color: #f70;
                    border-width: 5px;
                    border-radius: 10px;
                    height: 86%;
                    width: 96%;

                    VerticalLayout {

                        HorizontalLayout {
                            spacing: 5px;
                            padding: 3px;
                            alignment: start;
                            Rectangle {
                                    width: 100px;
                                    Text {
                                        color: #000;
                                        font-weight: 800;
                                        text: "SIZE";
                                        horizontal-alignment: right;
                                    }
                            }
                            Rectangle {
                                width: 100px;
                                Text {
                                    text: " ";
                                }
                            }
                            Text {
                                    color: #000;
                                    font-weight: 800;
                                    text: "PATHNAME";
                            }
                        }


                        for data[idx] in root.rows:
                            my-repeated-text := HorizontalLayout {
                                spacing: 5px;
                                padding: 3px;
                                alignment: start;
                                Rectangle {
                                    width: 100px;
                                    Text {
                                        color: #00f;
                                        text: data.size;
                                        horizontal-alignment: right;
                                    }
                                }
                                Rectangle {
                                    width: 100px;
                                    if data.directory: Button {
                                        clicked => { root.changeDirectory(data.path) }
                                        text: idx == 0 ? "\u{1F5D8}" : ">";
                                    }
                                    if !data.directory: Text {
                                        text: " ";
                                    }
                                }
                                Text {
                                    color: #000;
                                    text: data.path;
                                }
                            }

                    }
                }

            }
        }
}
