use crate::diff;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::command::Command,
    guild::Permissions,
    id::{marker::ChannelMarker, Id},
};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "tcone",
    desc = "Test command 2",
    default_permissions = "Self::permissions"
)]
#[allow(dead_code)]
struct TestCmd1 {
    /// The message to send.
    #[command(min_length = 1, max_length = 2000)]
    message: String,
    /// The text to put on the button
    #[command(min_length = 1, max_length = 32)]
    button_msg: String,
    /// The channel to send the message in
    button_channel: Id<ChannelMarker>,
    /// The channel to create modmails in
    modmail_channel: Id<ChannelMarker>,
}

impl TestCmd1 {
    const fn permissions() -> Permissions {
        Permissions::ADMINISTRATOR
    }
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "tc2", desc = "Test command 2")]
#[allow(dead_code)]
struct TestCmd2 {
    /// The message to send.
    #[command(min_length = 1, max_length = 2000)]
    message: String,
    /// The text to put on the button
    #[command(min_length = 1, max_length = 32)]
    button_msg: String,
    /// The channel to send the message in
    button_channel: Id<ChannelMarker>,
    /// The channel to create modmails in
    modmail_channel: Id<ChannelMarker>,
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "tc2", desc = "Test command 2")]
#[allow(dead_code)]
struct TestCmd2Update {
    /// The message to send.
    #[command(min_length = 1, max_length = 2000)]
    message: String,
}

fn with_id(cmd: Command, id: u64) -> Command {
    Command {
        id: Some(Id::new(id)),
        ..cmd
    }
}

#[test]
fn basic_add() {
    let d = diff(&[], &[TestCmd1::create_command().into()]);
    assert_eq!(d.to_create, &[TestCmd1::create_command().into()]);
    assert!(d.to_update.is_empty());
    assert!(d.to_delete.is_empty());
}

#[test]
fn basic_add_plus_delete() {
    let d = diff(
        &[with_id(TestCmd2::create_command().into(), 1)],
        &[TestCmd1::create_command().into()],
    );
    eprintln!("{d:#?}");
    assert_eq!(d.to_create, &[TestCmd1::create_command().into()]);
    assert!(d.to_update.is_empty());
    assert_eq!(d.to_delete, &[Id::new(1)]);
}

#[test]
fn different_options() {
    let d = diff(
        &[with_id(TestCmd2::create_command().into(), 1)],
        &[TestCmd2Update::create_command().into()],
    );
    eprintln!("{d:#?}");
    assert_eq!(d.to_update, &[(Id::new(1), TestCmd2Update::create_command().into())]);
    assert!(d.to_create.is_empty());
    assert!(d.to_delete.is_empty());
}