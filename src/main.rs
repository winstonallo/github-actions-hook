#![allow(unused)]

use saphyr::{LoadableYamlNode, Yaml};
use std::collections::BTreeMap;
use std::io::prelude::*;
use std::process::Command;
use std::{fs, io};

#[derive(Debug)]
struct Event {
    branches: Option<Vec<String>>,
    paths: Vec<String>,
}

#[derive(Debug)]
struct Step {
    name: String,
    command: String,
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

fn parse_action(path: &str) -> GithubActionWorkflow {
    let mut f = fs::File::open(path).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();

    let docs = Yaml::load_from_str(&s).unwrap();
    let mut name = String::new();
    let mut on = BTreeMap::<String, Event>::new();
    let mut jobs = BTreeMap::<String, Job>::new();
    let Yaml::Mapping(doc) = &docs[0] else { unreachable!() };
    for (k, v) in doc {
        if let Some(s) = k.as_str() {
            match s {
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
                                Some(s) => {
                                    if s == "steps" {
                                        let Yaml::Sequence(steps) = v else { unreachable!() };
                                        for step in steps {
                                            if let Some(command) = step.as_mapping_get("run")
                                                && !jobs.contains_key(job_name.as_str().unwrap())
                                            {
                                                jobs.insert(
                                                    job_name.as_str().unwrap().to_owned(),
                                                    Job {
                                                        steps: Vec::from([command.as_str().unwrap().to_owned()]),
                                                    },
                                                );
                                            };
                                        }
                                    }
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
    GithubActionWorkflow { name, on, jobs }
}

struct Failure {
    command: String,
    exit_code: i32,
    stderr: String,
    stdout: String,
}

fn main() -> io::Result<()> {
    let run_id = uuid::Uuid::new_v4().to_string();
    let mut workflows = Vec::new();
    for file in fs::read_dir(".github/workflows")? {
        workflows.push(parse_action(file.unwrap().path().to_str().unwrap()));
    }
    let mut failures = Vec::new();
    for workflow in workflows {
        for job in workflow.jobs {
            for step in job.1.steps {
                let stdout_path = format!("/tmp/{run_id}_{}")
                println!("Running {step}");
                let mut command = Command::new("sh");
                command.arg("-c").arg(&step);
                let mut child = command.output()?;
                let Some(exit_code) = child.status.code() else { panic!("No exit code") };
                println!("Exit status: {exit_code}");
                if exit_code != 0 {
                    failures.push(Failure { command: step, exit_code });
                }
            }
        }
    }
    Ok(())
}
