mod model;
mod state;

use youth_sdk::prelude::*;

struct Todo;

impl Application for Todo {
    fn view(context: &ViewContext) -> Result<Tree> {
        let loaded = state::load(&context.state())?;
        Ok(Tree::root(Column::new([
            Text::new(
                node!("summary"),
                format!(
                    "{} active / {} total",
                    loaded.model.active_count(),
                    loaded.model.items.len()
                ),
            ),
            Text::new(
                node!("blocker"),
                "Gate A: published SDK has no dynamic identities or structural updates",
            ),
            Button::command(command!("add"), "Add task"),
        ])))
    }

    fn handle(_context: &mut EventContext, events: &Events) -> Result<Update> {
        if events.commanded(command!("add")) {
            return Err(Error::rejected_event().with_message(
                "Gate A cannot create a stable dynamic row with the published SDK",
            ));
        }
        Ok(Update::unchanged())
    }
}

youth_sdk::export_app!(Todo);
