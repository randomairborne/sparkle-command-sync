#![warn(clippy::all, clippy::nursery, clippy::pedantic)]
use std::{cmp::Ordering, collections::HashMap};

use twilight_http::{request::Request, Client};
use twilight_model::{
    application::command::{Command, CommandOption, CommandOptionChoice},
    channel::ChannelType,
    id::{
        marker::{ApplicationMarker, CommandMarker},
        Id,
    },
};

#[cfg(test)]
mod test;

#[must_use]
pub fn diff(existing_commands_list: &[Command], desired_commands: &[Command]) -> CommandDiff {
    let mut existing_commands = HashMap::new();

    for command in existing_commands_list {
        existing_commands.insert((command.kind, command.name.clone()), command);
    }

    let to_delete: Vec<Id<CommandMarker>> = existing_commands
        .iter()
        .filter_map(|(existing_cmd_key, existing_cmd)| {
            if desired_commands.iter().any(|desired_cmd| {
                (desired_cmd.kind, desired_cmd.name.clone()) == *existing_cmd_key
            }) {
                None
            } else {
                // this is already an option, and if we don't have it we can't really delete the command
                existing_cmd.id
            }
        })
        .collect();

    let to_create: Vec<Command> = desired_commands
        .iter()
        // if we already have the command, we don't need to recreate it
        .filter(|desired_cmd| {
            !existing_commands.contains_key(&(desired_cmd.kind, desired_cmd.name.clone()))
        })
        .cloned()
        .collect();

    let to_update = desired_commands
        .iter()
        .filter_map(|desired_cmd| {
            let Some(existing) =
                existing_commands.get(&(desired_cmd.kind, desired_cmd.name.clone()))
            else {
                // We already added this to the to_create list, remove it from the update list
                return None;
            };
            let Some(existing_id) = existing.id else {
                // if we don't know the ID, we can't do anything
                return None;
            };

            if command_eq(desired_cmd, existing) {
                None
            } else {
                Some((existing_id, desired_cmd.clone()))
            }
        })
        .collect();

    CommandDiff {
        to_delete,
        to_create,
        to_update,
    }
}

fn command_eq(a: &Command, b: &Command) -> bool {
    a.name == b.name
        && a.kind == b.kind
        && a.description == b.description
        && a.default_member_permissions == b.default_member_permissions
        && localizations_eq(a.name_localizations.as_ref(), a.name_localizations.as_ref())
        && localizations_eq(
            a.description_localizations.as_ref(),
            b.description_localizations.as_ref(),
        )
        && a.dm_permission.unwrap_or(false) == b.dm_permission.unwrap_or(false)
        && a.nsfw.unwrap_or(false) == b.nsfw.unwrap_or(false)
        && options_eq(&a.options, &b.options)
}

fn localizations_eq(
    a: Option<&HashMap<String, String>>,
    b: Option<&HashMap<String, String>>,
) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => a == b,
        (Some(a), None) | (None, Some(a)) => a.is_empty(),
        (None, None) => true,
    }
}

fn options_eq(a: &[CommandOption], b: &[CommandOption]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let a = a.to_vec().sorted_by(|a, b| a.name.cmp(&b.name));
    let b = b.to_vec().sorted_by(|a, b| a.name.cmp(&b.name));
    std::iter::zip(a, b).all(cmd_opt_eq)
}

fn cmd_opt_eq((a, b): (CommandOption, CommandOption)) -> bool {
    a.name == b.name
        && a.kind == b.kind
        && a.autocomplete.unwrap_or(false) == b.autocomplete.unwrap_or(false)
        && channel_types_eq(
            &a.channel_types.unwrap_or_else(Vec::new),
            &b.channel_types.unwrap_or_else(Vec::new),
        )
        && choices_eq(
            &a.choices.unwrap_or(Vec::new()),
            &b.choices.unwrap_or(Vec::new()),
        )
        && a.description == b.description
        && localizations_eq(
            a.description_localizations.as_ref(),
            b.description_localizations.as_ref(),
        )
        && a.max_length == b.max_length
        && a.max_value == b.max_value
        && a.min_length == b.min_length
        && a.min_value == b.min_value
        && localizations_eq(a.name_localizations.as_ref(), b.name_localizations.as_ref())
        && options_eq(
            &a.options.unwrap_or_else(Vec::new),
            &b.options.unwrap_or_else(Vec::new),
        )
        && a.required.unwrap_or(false) == b.required.unwrap_or(false)
}

fn choices_eq(a: &[CommandOptionChoice], b: &[CommandOptionChoice]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let a = a.to_vec().sorted_by(|ae, be| ae.name.cmp(&be.name));
    let b = b.to_vec().sorted_by(|ae, be| ae.name.cmp(&be.name));
    std::iter::zip(a, b).all(cmd_opt_choice_eq)
}

fn cmd_opt_choice_eq((a, b): (CommandOptionChoice, CommandOptionChoice)) -> bool {
    a.name == b.name
        && localizations_eq(a.name_localizations.as_ref(), b.name_localizations.as_ref())
        && a.value == b.value
}

fn channel_types_eq(a: &[ChannelType], b: &[ChannelType]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let a = a.to_vec().sorted_by(|a, b| cmp_channel_types(*a, *b));
    let b = b.to_vec().sorted_by(|a, b| cmp_channel_types(*a, *b));
    a == b
}

fn cmp_channel_types(a: ChannelType, b: ChannelType) -> Ordering {
    let a: u8 = a.into();
    let b: u8 = b.into();
    a.cmp(&b)
}

#[derive(Debug, Clone)]
pub struct CommandDiff {
    pub to_delete: Vec<Id<CommandMarker>>,
    pub to_create: Vec<Command>,
    pub to_update: Vec<(Id<CommandMarker>, Command)>,
}

/// Sync your list of desired commands with discord, overwriting on errors.
/// # Errors
/// This function can error if your commands are invalid.
pub async fn sync<'a>(
    client: Client,
    application_id: Id<ApplicationMarker>,
    desired: &[Command],
) -> Result<(), Error> {
    let iclient = client.interaction(application_id);
    let command_list = iclient.global_commands().await?.model().await?;

    let diff = diff(&command_list, desired);

    for command_id in diff.to_delete {
        iclient.delete_global_command(command_id).await?;
    }

    for command in diff.to_create {
        let create_req = Request::builder(&twilight_http::routing::Route::CreateGlobalCommand {
            application_id: application_id.into(),
        })
        .json(&command)
        .build()?;
        client.request::<Command>(create_req).await?.model().await?;
    }

    for command in diff.to_update {
        let create_req = Request::builder(&twilight_http::routing::Route::UpdateGlobalCommand {
            application_id: application_id.into(),
            command_id: command.0.get(),
        })
        .json(&command)
        .build()?;
        client.request::<Command>(create_req).await?.model().await?;
    }

    Ok(())
}

pub trait Sorted {
    #[must_use]
    fn sorted(self) -> Self;
}

pub trait SortedBy<T> {
    #[must_use]
    fn sorted_by<F>(self, f: F) -> Self
    where
        F: for<'a, 'b> FnMut(&'a T, &'b T) -> std::cmp::Ordering;
}

impl<T> Sorted for Vec<T>
where
    T: PartialOrd + Ord,
{
    fn sorted(mut self) -> Self {
        self.sort();
        self
    }
}

impl<T> SortedBy<T> for Vec<T> {
    fn sorted_by<F: for<'a, 'b> FnMut(&'a T, &'b T) -> std::cmp::Ordering>(mut self, f: F) -> Self {
        self.sort_by(f);
        self
    }
}

#[derive(Debug)]
pub enum Error {
    Http(twilight_http::Error),
    Body(twilight_http::response::DeserializeBodyError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Body(be) => write!(f, "body error: {be}"),
            Self::Http(he) => write!(f, "http error: {he}"),
        }
    }
}

impl From<twilight_http::Error> for Error {
    fn from(value: twilight_http::Error) -> Self {
        Self::Http(value)
    }
}

impl From<twilight_http::response::DeserializeBodyError> for Error {
    fn from(value: twilight_http::response::DeserializeBodyError) -> Self {
        Self::Body(value)
    }
}

impl std::error::Error for Error {}
