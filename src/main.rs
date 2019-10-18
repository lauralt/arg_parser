use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

const DEFAULT_API_SOCK_PATH: &str = "/tmp/firecracker.socket";
const DEFAULT_INSTANCE_ID: &str = "anonymous-instance";

struct ArgumentInfo<'a> {
    name: &'a str,
    required: bool,
    conflicts_with: Option<&'a str>,
    requires: Option<&'a str>,
    takes_value: bool,
    default_value: Option<&'a str>,
    help: &'a str,
}

impl<'a> ArgumentInfo<'a> {
    pub fn new(
        name: &'a str,
        required: bool,
        conflicts_with: Option<&'a str>,
        requires: Option<&'a str>,
        takes_value: bool,
        default_value: Option<&'a str>,
        help: &'a str,
    ) -> ArgumentInfo<'a> {
        ArgumentInfo {
            name,
            required,
            conflicts_with,
            requires,
            takes_value,
            default_value,
            help,
        }
    }
    pub fn display_help(&self) {
        println!("{}: {}", self.name, self.help);
    }
}

fn check_is_valid(argument_info: &ArgumentInfo, params: &Vec<&str>) {
    if let Some(arg_name) = argument_info.conflicts_with {
        if params.contains(&arg_name) {
            panic!(
                "Found argument '{}' which wasn't expected, or isn't valid in this context.",
                arg_name
            );
        }
    }
    if let Some(arg_name) = argument_info.requires {
        if !params.contains(&arg_name) {
            panic!("Argument '{}' required, but not found.", arg_name);
        }
    }
    if argument_info.takes_value {
        if argument_info.required && argument_info.default_value.is_none() {
            if !params.contains(&argument_info.name) {
                panic!("Argument '{}' required, but not found.", argument_info.name);
            }
        }
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let args: Vec<&str> = args.iter().map(|s| s as &str).collect();

    let params: Vec<&str> = args
        .iter()
        .filter(|x| x.starts_with("--"))
        .map(|&x| x.trim_start_matches("--"))
        .collect();

    let fc_extra_args: Vec<&str> = if params.last().unwrap().is_empty() {
        let index = args.iter().position(|&r| r == "--").unwrap();
        let (_, extra_args) = args.split_at(index + 1);
        extra_args.to_vec()
    } else {
        Vec::new()
    };

    for extra_arg in fc_extra_args {
        println!("{}", extra_arg);
    }

    let mut fc_args = Vec::new();
    let fc_args_str = [
        "api-sock",
        "id",
        "seccomp-level",
        "start-time-us",
        "start-time-cpu-us",
        "no-api",
        "config-file",
    ];
    let seccomp_values = ["0", "1", "2"];
    let mut values = HashMap::new();

    fc_args.push(ArgumentInfo::new(
        "api-sock",
        false,
        None,
        None,
        true,
        Some(DEFAULT_API_SOCK_PATH),
        "Path to unix domain socket used by the API",
    ));
    fc_args.push(ArgumentInfo::new(
        "id",
        false,
        None,
        None,
        true,
        Some(DEFAULT_INSTANCE_ID),
        "MicroVM unique identifier",
    ));
    fc_args.push(ArgumentInfo::new(
        "seccomp-level",
        false,
        None,
        None,
        true,
        Some("2"),
        "Level of seccomp filtering.\n
                            - Level 0: No filtering.\n
                            - Level 1: Seccomp filtering by syscall number.\n
                            - Level 2: Seccomp filtering by syscall number and argument values.\n",
    ));
    fc_args.push(ArgumentInfo::new(
        "start-time-us",
        false,
        None,
        None,
        true,
        None,
        "",
    ));
    fc_args.push(ArgumentInfo::new(
        "start-time-cpu-us",
        false,
        None,
        None,
        true,
        None,
        "",
    ));
    fc_args.push(ArgumentInfo::new(
        "no-api",
        false,
        None,
        Some("config-file"),
        false,
        None,
        "Optional parameter which allows starting and using a microVM without an active API socket.",
    ));
    fc_args.push(ArgumentInfo::new(
        "config-file",
        false,
        None,
        None,
        true,
        None,
        "Path to a file that contains the microVM configuration in JSON format.",
    ));
    fc_args.push(ArgumentInfo::new(
        "extra-args",
        false,
        None,
        None,
        true,
        None,
        "Arguments that will be passed verbatim to the exec file.",
    ));

    if params.contains(&"help") {
        println!("Stuff about firecracker");
        for arg in fc_args {
            arg.display_help();
        }
        return;
    }

    for (i, &item) in args.iter().enumerate() {
        if item.starts_with("--") && item != "--" {
            for argument_info in fc_args.iter() {
                let name = item.trim_start_matches("--");
                if !fc_args_str.contains(&name) {
                    panic!("Found argument '{}' which wasn't expected, or isn't valid in this context.", name);
                }
                if argument_info.name == name {
                    check_is_valid(argument_info, &params);
                    if argument_info.takes_value {
                        if args.get(i + 1).is_some() {
                            if let Some(&x) = args.get(i + 2) {
                                if !x.starts_with("--") {
                                    panic!("Found argument '{}' which wasn't expected, or isn't valid in this context.", x);
                                }
                            }
                        }
                        let param_value = args.get(i + 1).map(|x| *x).unwrap();
                        if name == "seccomp-level" && !seccomp_values.contains(&param_value) {
                            panic!(
                                "'{}' isn't a valid value for 'seccomp-level'. Must  be 0, 1 or 2.",
                                param_value
                            );
                        }
                        values.insert(name, param_value);
                        println!("{} {:?}", item, values.get(name));
                    } else {
                        if let Some(&x) = args.get(i + 1) {
                            if !x.starts_with("--") {
                                panic!("Found argument '{}' which wasn't expected, or isn't valid in this context.", x);
                            }
                        }
                        values.insert(name, "");
                        println!("{} {:?}", item, values.get(name));
                    }
                }
            }
        }
    }
    for arg in fc_args.iter() {
        if arg.required && !params.contains(&arg.name) {
            values.insert(arg.name, arg.default_value.unwrap());
        }
    }

    // checking that map values are accessible
    let _bind_path = values
        .get("api-sock")
        .map(PathBuf::from)
        .expect("Missing argument: api-sock");
}


