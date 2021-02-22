/// Custom event handlers
use crate::{Cmd, EditMode, InputMode, InputState, KeyEvent, Refresher, RepeatCount};

use radix_trie::TrieKey;
use serde::Serialize;
use smallvec::{smallvec, SmallVec};

/// Input event
#[derive(Debug, PartialEq, Eq, Hash, Serialize)]
pub enum Event {
    /// Wildcard.
    /// Useful if you want to filter out some keys.
    Any,
    /// Key sequence
    // TODO Validate 2 ?
    KeySeq(SmallVec<[KeyEvent; 2]>),
    /// TODO Mouse event
    Mouse(),
}

impl Event {
    /// See [`KeyEvent::normalize`]
    pub(crate) fn normalize(mut self) -> Self {
        if let Event::KeySeq(ref mut keys) = self {
            for key in keys.iter_mut() {
                *key = KeyEvent::normalize(*key);
            }
        }
        self
    }
}

impl From<KeyEvent> for Event {
    fn from(k: KeyEvent) -> Event {
        Event::KeySeq(smallvec![k])
    }
}

impl TrieKey for Event {
    fn encode_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

/// Event handler
pub enum EventHandler {
    /// unconditional command
    Simple(Cmd),
    /// handler behaviour depends on input state
    Conditional(Box<dyn ConditionalEventHandler>),
    /* invoke multiple actions
     * TODO Macro(), */
}

impl From<Cmd> for EventHandler {
    fn from(c: Cmd) -> EventHandler {
        EventHandler::Simple(c)
    }
}

/// Give access to user input.
pub struct EventContext<'r> {
    mode: EditMode,
    input_mode: InputMode,
    wrt: &'r dyn Refresher,
}

impl<'r> EventContext<'r> {
    pub(crate) fn new(is: &InputState, wrt: &'r dyn Refresher) -> Self {
        EventContext {
            mode: is.mode,
            input_mode: is.input_mode,
            wrt,
        }
    }

    /// emacs or vi mode
    pub fn mode(&self) -> EditMode {
        self.mode
    }

    /// vi input mode
    pub fn input_mode(&self) -> InputMode {
        self.input_mode
    }

    /// Returns `true` if there is a hint displayed.
    pub fn has_hint(&self) -> bool {
        self.wrt.has_hint()
    }

    /// currently edited line
    pub fn line(&self) -> &str {
        self.wrt.line()
    }

    /// Current cursor position (byte position)
    pub fn pos(&self) -> usize {
        self.wrt.pos()
    }
}

/// May behave differently depending on:
///  * edit mode (emacs vs vi)
///  * vi input mode (insert vs replace vs command modes)
///  * empty line
///  * cursor position
///  * repeat count
///  * original key pressed (when same command is bound to different key)
///  * hint
///  * ...
pub trait ConditionalEventHandler: Send + Sync {
    /// Takes the current input state and
    /// returns the command to be performed or `None` to perform the default
    /// one.
    fn handle(
        &self,
        evt: &Event,
        n: RepeatCount,
        positive: bool,
        ctx: &EventContext,
    ) -> Option<Cmd>;
}

#[cfg(test)]
mod test {
    use crate::{KeyEvent, Cmd};
    use super::{Event, EventHandler};
    use radix_trie::Trie;
    use smallvec::smallvec;

    #[test]
    fn encode() {
        let mut trie = Trie::new();
        let evt = Event::KeySeq(smallvec![KeyEvent::ctrl('X'), KeyEvent::ctrl('E')]);
        trie.insert(evt, EventHandler::from(Cmd::Noop));
        let prefix = Event::from(KeyEvent::ctrl('X'));
        let subtrie = trie.subtrie(&prefix);
        assert!(subtrie.is_some())
    }
}