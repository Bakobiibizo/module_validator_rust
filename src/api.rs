use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use regex::Regex;
use crate::inference::python_executor::PythonExecutor;

#[derive(Deserialize)]
struct SubnetCommandRequest {
    subnet: String,
    command: String,
    args: HashMap<String, serde_json::Value>,
}

#[derive(Serialize)]
struct ApiResponse {
    message: String,
}

struct SubnetCommand {
    name: String,
    args: Vec<String>,
}

fn discover_subnets() -> Vec<String> {
    let subnets_dir = Path::new("subnets");
    fs::read_dir(subnets_dir)
        .unwrap()
        .filter_map(|entry| {
            entry.ok().and_then(|e| 
                e.path().file_name()
                    .and_then(|n| n.to_str().map(String::from))
            )
        })
        .collect()
}

fn parse_subnet_commands(subnet: &str) -> Vec<SubnetCommand> {
    let subnet_dir = Path::new("subnets").join(subnet);
    let command_regex = Regex::new(r#"@app\.command\(['"](\w+)['"])\s*def\s+(\w+)\((.*?)\):"#).unwrap();
    let arg_regex = Regex::new(r"(\w+):\s*\w+(?:\s*=\s*[^,)]+)?").unwrap();

    fs::read_dir(subnet_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "py"))
        .flat_map(|entry| {
            let content = fs::read_to_string(entry.path()).unwrap();
            command_regex.captures_iter(&content)
                .map(|cap| {
                    let name = cap[1].to_string();
                    let args = arg_regex.captures_iter(&cap[3])
                        .map(|arg_cap| arg_cap[1].to_string())
                        .collect();
                    SubnetCommand { name, args }
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

async fn execute_subnet_command(info: web::Json<SubnetCommandRequest>) -> impl Responder {
    let subnet = &info.subnet;
    let command = &info.command;
    
    let commands = parse_subnet_commands(subnet);
    if let Some(cmd) = commands.iter().find(|c| c.name == *command) {
        let mut args = Vec::new();
        for arg_name in &cmd.args {
            if let Some(value) = info.args.get(arg_name) {
                args.push(value.to_string());
            } else {
                return HttpResponse::BadRequest().json(ApiResponse {
                    message: format!("Missing argument: {}", arg_name),
                });
            }
        }

        let module_name = subnet.to_string();
        let module_type = "subnet".to_string();
        let target_script_path = format!("subnets/{}/src/communex/cli/{}.py", subnet, command);

        match PythonExecutor::new(module_name, module_type, target_script_path) {
            Ok(python_executor) => {
                match python_executor.run_command(args.join(" ")) {
                    Ok(result) => HttpResponse::Ok().json(ApiResponse {
                        message: format!("Command result: {}", result),
                    }),
                    Err(e) => HttpResponse::InternalServerError().json(ApiResponse {
                        message: format!("Error executing command: {}", e),
                    }),
                }
            },
            Err(e) => HttpResponse::InternalServerError().json(ApiResponse {
                message: format!("Error creating Python executor: {}", e),
            }),
        }
    } else {
        HttpResponse::NotFound().json(ApiResponse {
            message: format!("Command not found: {}", command),
        })
    }
}

pub struct API;

impl API {
    pub async fn start(host: String, port: u16) -> std::io::Result<()> {
        HttpServer::new(|| {
            App::new()
                .route("/subnet_command", web::post().to(execute_subnet_command))
        })
        .bind(format!("{}:{}", host, port))?
        .run()
        .await
    }
}