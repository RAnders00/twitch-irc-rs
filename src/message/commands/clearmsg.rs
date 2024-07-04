use crate::message::commands::IRCMessageParseExt;
use crate::message::{IRCMessage, ServerMessageParseError};
use chrono::{DateTime, Utc};
use fast_str::FastStr;

#[cfg(feature = "with-serde")]
use {serde::Deserialize, serde::Serialize};

/// Message for when a single message is deleted from chat.
///
/// The deleted message is identified by its `message_id`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "with-serde",
    derive(
        Serialize,
        Deserialize
    )
)]
pub struct ClearMsgMessage {
    /// Login name of the channel that the deleted message was posted in.
    pub channel_login: FastStr,
    // pub channel_id: FastStr,
    /// login name of the user that sent the original message that was deleted by this
    /// `CLEARMSG`.
    pub sender_login: FastStr,
    /// ID of the message that was deleted.
    pub message_id: FastStr,
    /// Text of the message that was deleted
    pub message_text: FastStr,
    /// Whether the deleted message was an action (`/me`)
    pub is_action: bool,
    /// server timestamp for the time when the delete command was executed.
    pub server_timestamp: DateTime<Utc>,

    /// The message that this `ClearMsgMessage` was parsed from.
    pub source: IRCMessage,
}

impl TryFrom<IRCMessage> for ClearMsgMessage {
    type Error = ServerMessageParseError;

    fn try_from(source: IRCMessage) -> Result<ClearMsgMessage, ServerMessageParseError> {
        if source.command != "CLEARMSG" {
            return Err(ServerMessageParseError::MismatchedCommand(source));
        }

        // example msg:
        // @login=alazymeme;room-id=;target-msg-id=3c92014f-340a-4dc3-a9c9-e5cf182f4a84;tmi-sent-ts=1594561955611 :tmi.twitch.tv CLEARMSG #pajlada :NIGHT CUNT
        // room-id is currently empty on all incoming messages, so we don't parse it
        // see https://github.com/twitchdev/issues/issues/163
        let (message_text, is_action) = source.try_get_message_text()?;

        Ok(ClearMsgMessage {
            channel_login: FastStr::from_ref(source.try_get_channel_login()?),
            sender_login: FastStr::from_ref(source.try_get_nonempty_tag_value("login")?),
            message_id: FastStr::from_ref(source.try_get_nonempty_tag_value("target-msg-id")?),
            server_timestamp: source.try_get_timestamp("tmi-sent-ts")?,
            message_text: FastStr::from_ref(message_text),
            is_action,
            source,
        })
    }
}

impl From<ClearMsgMessage> for IRCMessage {
    fn from(msg: ClearMsgMessage) -> IRCMessage {
        msg.source
    }
}

#[cfg(test)]
mod tests {
    use crate::message::{ClearMsgMessage, IRCMessage};
    use chrono::{TimeZone, Utc};
    use std::convert::TryFrom;

    #[test]
    pub fn test_simple() {
        let src = "@login=alazymeme;room-id=;target-msg-id=3c92014f-340a-4dc3-a9c9-e5cf182f4a84;tmi-sent-ts=1594561955611 :tmi.twitch.tv CLEARMSG #pajlada :NIGHT CUNT";
        let irc_message = IRCMessage::parse(src).unwrap();
        let msg = ClearMsgMessage::try_from(irc_message.clone()).unwrap();

        assert_eq!(
            msg,
            ClearMsgMessage {
                channel_login: "pajlada".into(),
                sender_login: "alazymeme".into(),
                message_id: "3c92014f-340a-4dc3-a9c9-e5cf182f4a84".into(),
                message_text: "NIGHT CUNT".into(),
                is_action: false,
                server_timestamp: Utc.timestamp_millis_opt(1594561955611).unwrap(),
                source: irc_message
            }
        )
    }

    #[test]
    pub fn test_action() {
        let src = "@login=randers;room-id=;target-msg-id=15e5164d-f8e6-4aec-baf4-2d6a330760c4;tmi-sent-ts=1594562632383 :tmi.twitch.tv CLEARMSG #pajlada :\u{0001}ACTION test\u{0001}";
        let irc_message = IRCMessage::parse(src).unwrap();
        let msg = ClearMsgMessage::try_from(irc_message.clone()).unwrap();

        assert_eq!(
            msg,
            ClearMsgMessage {
                channel_login: "pajlada".into(),
                sender_login: "randers".into(),
                message_id: "15e5164d-f8e6-4aec-baf4-2d6a330760c4".into(),
                message_text: "test".into(),
                is_action: true,
                server_timestamp: Utc.timestamp_millis_opt(1594562632383).unwrap(),
                source: irc_message
            }
        )
    }
}
