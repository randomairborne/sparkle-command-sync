use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{guild::Permissions, id::{marker::ChannelMarker, Id}};
use crate::{CommandDiff, diff};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "setup",
    desc = "Initialize the modmail form",
    dm_permission = false,
    default_permissions = "Self::permissions"
)]
#[allow(dead_code)]
struct SetupCommand {
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

impl SetupCommand {
    const fn permissions() -> Permissions {
        Permissions::ADMINISTRATOR
    }
}

#[test]
fn basic_add() {
    let d = diff(&[], &[SetupCommand::create_command().into()]);
    assert_eq!(d.to_create, &[SetupCommand::create_command().into()]);
    assert!(d.to_update.is_empty());
    assert!(d.to_delete.is_empty());
}