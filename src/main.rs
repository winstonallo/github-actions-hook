use saphyr::{LoadableYamlNode, Yaml};
use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
struct Event {
    branches: Option<Vec<String>>,
    paths: Vec<String>,
}

#[derive(Debug)]
struct Job {
    steps: Vec<String>,
}

#[derive(Debug)]
struct GithubActionWorkflow {
    name: String,
    on: BTreeMap<String, Event>,
    jobs: BTreeMap<String, Job>,
}

fn print_indent(indent: usize) {
    for _ in 0..indent {
        print!("{}   ", indent);
    }
}

fn dump_node(doc: &Yaml, indent: usize) {
    match *doc {
        Yaml::Sequence(ref v) => {
            for x in v {
                dump_node(x, indent + 1);
            }
        }
        Yaml::Mapping(ref h) => {
            for (k, v) in h {
                print_indent(indent);
                if v.contains_mapping_key("<<") {
                    println!("{}:", k.as_str().unwrap());
                    dump_node(v.as_mapping_get("<<").unwrap(), indent + 1);
                } else {
                    println!("{}:", k.as_str().unwrap());
                    dump_node(v, indent + 1);
                }
            }
        }
        _ => {
            print_indent(indent);
            println!("{}", doc.as_str().unwrap());
        }
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let mut f = File::open(&args[1]).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();

    let docs = Yaml::load_from_str(&s).unwrap();
    let workflow = {
        let mut name = String::new();
        let mut on = BTreeMap::<String, Event>::new();
        let mut jobs = BTreeMap::<String, Job>::new();
        let Yaml::Mapping(doc) = &docs[0] else { unreachable!() };
        for (k, v) in doc {
            match k.as_str() {
                Some(s) => match s {
                    "name" => name = s.to_owned(),
                    "on" => {
                        on.insert(
                            s.to_owned(),
                            Event {
                                branches: Some(Vec::new()),
                                paths: Vec::new(),
                            },
                        );
                    }
                    "jobs" => {
                        let Yaml::Mapping(jbs) = v else { panic!("'jobs' value is not a mapping") };
                        for (job_name, v) in jbs {
                            let Yaml::Mapping(job) = v else { unreachable!() };
                            for (k, v) in job {
                                match k.as_str() {
                                    Some(s) => match s {
                                        "steps" => {
                                            let Yaml::Sequence(steps) = v else { unreachable!() };
                                            for step in steps {
                                                match step.as_mapping_get("run") {
                                                    Some(command) => {
                                                        if jobs.get(job_name.as_str().unwrap()).is_none() {
                                                            jobs.insert(
                                                                job_name.as_str().unwrap().to_owned(),
                                                                Job {
                                                                    steps: Vec::from([command.as_str().unwrap().to_owned()]),
                                                                },
                                                            );
                                                        };
                                                    }
                                                    None => {}
                                                };
                                            }

                                            println!("{v:?}");
                                        }
                                        _ => {}
                                    },
                                    _ => unreachable!(),
                                }
                            }
                        }
                        // jobs.insert(s.to_owned(), Job { steps: Vec::new() });
                    }
                    _ => unreachable!(),
                },
                None => {}
            }

            // dump_node(doc, 0);
        }
        GithubActionWorkflow { name, on, jobs }
    };

    println!("{workflow:?}")
}
