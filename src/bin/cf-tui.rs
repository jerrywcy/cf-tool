use std::{collections::HashMap, fs::write, io, path::PathBuf};

use cf::{
    api::methods::contest_list,
    args::args,
    display::tui::{app::App, msg::ViewConstructor},
    log::setup_logger,
    settings::{get_config_file_path, load_settings, CFCommand, CFScripts, CFSettings, CFTemplate},
};
use clap::ArgMatches;
use color_eyre::{
    eyre::{bail, eyre, Context, ContextCompat},
    Result,
};

fn input_template() -> Result<CFTemplate> {
    println!("Please input alias of the template:");
    let mut alias = String::default();
    io::stdin().read_line(&mut alias)?;
    println!("Please input language of the template:");
    let mut lang = String::default();
    io::stdin().read_line(&mut lang)?;
    println!("Please input path to the template code:");
    let mut path = String::default();
    io::stdin().read_line(&mut path)?;
    let path = PathBuf::from(path);
    let template = CFTemplate {
        alias,
        lang,
        path: path.clone(),
    };
    if !path.try_exists()? {
        bail!(
            "Template code doesn't exist or cf-tool has no permission to it: {}",
            path.display()
        );
    }
    Ok(template)
}

fn input_command() -> Result<CFCommand> {
    println!("Please input extension of the template:");
    let mut ext = String::default();
    io::stdin().read_line(&mut ext)?;
    println!("Please input before_script of the template:");
    let mut before_script = String::default();
    io::stdin().read_line(&mut before_script)?;
    println!("Please input script of the template:");
    let mut script = String::default();
    io::stdin().read_line(&mut script)?;
    println!("Please input after_script of the template:");
    let mut after_script = String::default();
    io::stdin().read_line(&mut after_script)?;
    println!("Please input open_script of the template:(if this is empty, the default \"vim <% full %>\")");
    let mut open_script = String::default();
    io::stdin().read_line(&mut open_script)?;
    if open_script == "" {
        open_script = String::from("vim <% full %>");
    }
    let command = CFCommand {
        ext,
        before_command: before_script,
        command: script,
        after_command: after_script,
        open_command: open_script,
    };
    Ok(command)
}

fn config_login(settings: &mut CFSettings) -> Result<()> {
    println!("Please input your username:");
    let mut username = String::default();
    io::stdin().read_line(&mut username)?;
    println!("Please input your API key:");
    let mut key = String::default();
    io::stdin().read_line(&mut key)?;
    println!("Please input your API secret:");
    let mut secret = String::default();
    io::stdin().read_line(&mut secret)?;
    settings.username = Some(username);
    settings.key = Some(key);
    settings.secret = Some(secret);
    Ok(())
}

fn config_templates(settings: &mut CFSettings) -> Result<()> {
    println!(
        "Which of the following operation do you wanna do?\n\
        1. Add a template.\n\
        2. Delete a template.\n\
        3. Modify a template."
    );
    let mut operation = String::default();
    io::stdin().read_line(&mut operation)?;
    let operation: usize = operation
        .trim()
        .parse()
        .wrap_err("Please input an integer that is within 1 and 3.")?;
    match operation {
        1 => {
            let template = input_template()?;
            match &mut settings.templates {
                Some(templates) => templates.push(template),
                None => settings.templates = Some(vec![template]),
            };
        }
        2 => match &mut settings.templates {
            Some(templates) => {
                let len = templates.len();
                println!(
                    "Currently {} templates are configured:\n {}",
                    len,
                    serde_json::to_string_pretty(&templates)?
                );
                println!("Which template do you want to delete?");
                let mut index = String::default();
                io::stdin().read_line(&mut index)?;
                let index: usize = index.trim().parse().wrap_err(format!(
                    "Please input an integer that is within 1 and {len}."
                ))?;
                if index < 1 || index > len {
                    bail!("Please input an integer that is within 1 and {len}.");
                }
                templates.swap_remove(index - 1);
            }
            None => {
                println!("No template is configured!");
                return Ok(());
            }
        },
        3 => match &mut settings.templates {
            Some(templates) => {
                let len = templates.len();
                println!(
                    "Currently {} templates are configured:\n {}",
                    len,
                    serde_json::to_string_pretty(&templates)?
                );
                println!("Which template do you want to modify?");
                let mut index = String::default();
                io::stdin().read_line(&mut index)?;
                let index: usize = index.trim().parse().wrap_err(format!(
                    "Please input an integer that is within 1 and {len}."
                ))?;
                if index < 1 || index > len {
                    bail!("Please input an integer that is within 1 and {len}.");
                }

                let template = input_template()?;

                *templates
                    .get_mut(index - 1)
                    .wrap_err(format!("Template #{index} doesn't exist!"))? = template;
            }
            None => {
                println!("No template is configured!");
                return Ok(());
            }
        },

        _ => bail!("Please input an integer that is within 1 and 3."),
    }
    Ok(())
}

fn config_commands(settings: &mut CFSettings) -> Result<()> {
    println!(
        "Which of the following operation do you wanna do?\n\
        1. Add a command.\n\
        2. Delete a command.\n\
        3. Modify a command."
    );
    let mut operation = String::default();
    io::stdin().read_line(&mut operation)?;
    let operation: usize = operation
        .trim()
        .parse()
        .wrap_err("Please input an integer that is within 1 and 3.")?;
    match operation {
        1 => {
            let command = input_command()?;
            match &mut settings.commands {
                Some(commands) => {
                    commands.insert(command.ext.clone(), CFScripts::from(command));
                }
                None => {
                    let mut commands = HashMap::new();
                    commands.insert(command.ext.clone(), CFScripts::from(command));
                    settings.commands = Some(commands);
                }
            };
        }
        2 => match &mut settings.templates {
            Some(templates) => {
                let len = templates.len();
                println!(
                    "Currently {} templates are configured:\n {}",
                    len,
                    serde_json::to_string_pretty(&templates)?
                );
                println!("Which template do you want to delete?");
                let mut index = String::default();
                io::stdin().read_line(&mut index)?;
                let index: usize = index.trim().parse().wrap_err(format!(
                    "Please input an integer that is within 1 and {len}."
                ))?;
                if index < 1 || index > len {
                    bail!("Please input an integer that is within 1 and {len}.");
                }
                templates.swap_remove(index - 1);
            }
            None => {
                println!("No template is configured!");
                return Ok(());
            }
        },
        3 => match &mut settings.templates {
            Some(templates) => {
                let len = templates.len();
                println!(
                    "Currently {} templates are configured:\n {}",
                    len,
                    serde_json::to_string_pretty(&templates)?
                );
                println!("Which template do you want to modify?");
                let mut index = String::default();
                io::stdin().read_line(&mut index)?;
                let index: usize = index.trim().parse().wrap_err(format!(
                    "Please input an integer that is within 1 and {len}."
                ))?;
                if index < 1 || index > len {
                    bail!("Please input an integer that is within 1 and {len}.");
                }

                let template = input_template()?;

                *templates
                    .get_mut(index - 1)
                    .wrap_err(format!("Template #{index} doesn't exist!"))? = template;
            }
            None => {
                println!("No template is configured!");
                return Ok(());
            }
        },

        _ => bail!("Please input an integer that is within 1 and 3."),
    }
    Ok(())
}

fn handle_config(matches: &ArgMatches) -> Result<()> {
    let mut settings = load_settings()?;
    match matches.subcommand() {
        Some(("login", _)) => {
            config_login(&mut settings)?;
        }
        Some(("templates", _)) => {
            config_templates(&mut settings)?;
        }
        Some(("commands", _)) => {
            config_commands(&mut settings)?;
        }
        _ => unreachable!(),
    };
    let config_file_path = get_config_file_path()?;
    if !config_file_path.try_exists()? {
        bail!(
            "Config directory doesn't exist or cf-tool has no permission to it: {}",
            config_file_path.display()
        );
    }
    write(config_file_path, serde_json::to_string_pretty(&settings)?)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_logger()?;
    let args = args().get_matches();
    match args.subcommand() {
        Some(("config", sub_matches)) => {
            handle_config(sub_matches)?;
        }
        Some(("race", sub_matches)) => {
            let contest_id = sub_matches
                .get_one::<i32>("CONTEST_ID")
                .ok_or(eyre!("Cannot find contest_id"))?;
            let contest_list = contest_list(None).await?;
            for contest in contest_list {
                if contest.id == *contest_id {
                    let mut app = App::new()?;
                    app.enter_new_view(ViewConstructor::ContestBrowser(contest));
                    if let Err(err) = app.run() {
                        drop(app);
                        eprintln!("{err:#}");
                    }
                    break;
                }
            }
        }
        _ => {
            let mut app = App::new()?;
            app.enter_new_view(ViewConstructor::MainBrowser);
            if let Err(err) = app.run() {
                drop(app);
                eprintln!("{err:#}");
            }
        }
    }

    Ok(())
}
