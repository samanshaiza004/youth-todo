#![cfg_attr(not(all(target_os = "wasi", target_env = "p2")), allow(dead_code))]

mod model;
mod state;

use std::cell::RefCell;

use model::{Filter, MAX_ITEMS, ModelError, Status, TaskId, TodoItem, TodoModel, TodoSession};
use youth_sdk::prelude::*;

thread_local! {
    static SESSION: RefCell<TodoSession> = RefCell::new(TodoSession::default());
}

struct Todo;

#[derive(Clone, Debug, Eq, PartialEq)]
enum CommandAction {
    Add,
    ClearCompleted,
    SetFilter(Filter),
    Previous,
    Next,
    Toggle(TaskId),
    Delete(TaskId),
    MoveUp(TaskId),
    MoveDown(TaskId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PresentationOperation {
    Remove {
        parent: NodeIdentity,
        child: NodeIdentity,
    },
    Move {
        parent: NodeIdentity,
        child: NodeIdentity,
        index: usize,
    },
    Insert {
        parent: NodeIdentity,
        index: usize,
        subtree: Element,
    },
    Text(NodeIdentity, String),
    Label(NodeIdentity, String),
    Enabled(NodeIdentity, bool),
}

impl Application for Todo {
    fn view(context: &ViewContext) -> Result<Tree> {
        let loaded = state::load(&context.state())?;
        let session = SESSION.with(|session| *session.borrow());
        build_tree(&loaded.model, session)
    }

    fn handle(context: &mut EventContext, events: &Events) -> Result<Update> {
        let loaded = state::load(&context.state())?;
        let old_model = loaded.model;
        let old_session = SESSION.with(|session| *session.borrow());
        let action = requested_action(events, &old_model)?;
        let mut new_model = old_model.clone();
        let mut new_session = old_session;
        apply_action(&mut new_model, &mut new_session, action).map_err(app_error)?;
        new_session.clamp(&new_model);

        if loaded.needs_migration || new_model != old_model {
            let mut writer = context.state();
            state::persist(&mut writer, &old_model, &new_model, loaded.needs_migration)?;
        }
        let update = into_update(presentation_diff(
            &old_model,
            old_session,
            &new_model,
            new_session,
        )?);
        SESSION.with(|session| *session.borrow_mut() = new_session);
        Ok(update)
    }
}

fn requested_action(events: &Events, model: &TodoModel) -> Result<CommandAction> {
    let static_commands = [
        (command!("add"), CommandAction::Add),
        (command!("clear-completed"), CommandAction::ClearCompleted),
        (
            command!("filter-all"),
            CommandAction::SetFilter(Filter::All),
        ),
        (
            command!("filter-active"),
            CommandAction::SetFilter(Filter::Active),
        ),
        (
            command!("filter-completed"),
            CommandAction::SetFilter(Filter::Completed),
        ),
        (command!("previous"), CommandAction::Previous),
        (command!("next"), CommandAction::Next),
    ];
    for (command, action) in static_commands {
        if events.commanded(command) {
            return Ok(action);
        }
    }
    for item in model.ordered() {
        let key = item_key(item.id)?;
        for (role, action) in [
            ("toggle", CommandAction::Toggle(item.id)),
            ("delete", CommandAction::Delete(item.id)),
            ("up", CommandAction::MoveUp(item.id)),
            ("down", CommandAction::MoveDown(item.id)),
        ] {
            if events.commanded(key.command(role)?) {
                return Ok(action);
            }
        }
    }
    Err(Error::rejected_event().with_message("Todo does not recognize this command"))
}

fn apply_action(
    model: &mut TodoModel,
    session: &mut TodoSession,
    action: CommandAction,
) -> std::result::Result<(), ModelError> {
    match action {
        CommandAction::Add => {
            model.add()?;
        }
        CommandAction::ClearCompleted => {
            model.clear_completed()?;
        }
        CommandAction::SetFilter(filter) => session.set_filter(filter)?,
        CommandAction::Previous => session.previous()?,
        CommandAction::Next => session.next(model)?,
        CommandAction::Toggle(id) => model.toggle(id)?,
        CommandAction::Delete(id) => model.delete(id)?,
        CommandAction::MoveUp(id) => {
            if session.filter != Filter::All {
                return Err(ModelError::Rejected(
                    "reordering is available only under All",
                ));
            }
            model.move_by(id, -1)?;
        }
        CommandAction::MoveDown(id) => {
            if session.filter != Filter::All {
                return Err(ModelError::Rejected(
                    "reordering is available only under All",
                ));
            }
            model.move_by(id, 1)?;
        }
    }
    Ok(())
}

fn build_tree(model: &TodoModel, session: TodoSession) -> Result<Tree> {
    let items = visible_children(model, session)?;
    Ok(Tree::root(Column::new([
        Text::new(node!("summary"), summary(model)),
        Row::new([
            Button::command(command!("filter-all"), "All").enabled(session.filter != Filter::All),
            Button::command(command!("filter-active"), "Active")
                .enabled(session.filter != Filter::Active),
            Button::command(command!("filter-completed"), "Completed")
                .enabled(session.filter != Filter::Completed),
        ]),
        Row::new([
            Button::command(command!("add"), "Add").enabled(can_add(model)),
            Button::command(command!("clear-completed"), "Clear Completed")
                .enabled(has_completed(model)),
        ]),
        Text::new(node!("page"), page_label(model, session)),
        Column::named(node!("items"), items),
        Row::new([
            Button::command(command!("previous"), "Previous").enabled(session.page > 0),
            Button::command(command!("next"), "Next")
                .enabled(session.page + 1 < session.page_count(model)),
        ]),
    ])))
}

fn visible_children(model: &TodoModel, session: TodoSession) -> Result<Vec<Element>> {
    let visible = session.visible_ids(model);
    if visible.is_empty() {
        return Ok(vec![Text::new(node!("empty"), "No tasks")]);
    }
    visible
        .into_iter()
        .map(|id| {
            row(
                model.items.get(&id).expect("visible task exists"),
                model,
                session,
            )
        })
        .collect()
}

fn row(item: &TodoItem, model: &TodoModel, session: TodoSession) -> Result<Element> {
    let key = item_key(item.id)?;
    let index = model
        .order
        .iter()
        .position(|id| *id == item.id)
        .expect("item occurs in canonical order");
    let reorder = session.filter == Filter::All;
    Ok(Row::named(
        key.node("row")?,
        [
            Text::new(
                key.node("status")?,
                if item.status == Status::Completed {
                    "Completed"
                } else {
                    "Active"
                },
            ),
            Text::new(key.node("title")?, item.title.clone()),
            Button::command(
                key.command("toggle")?,
                if item.status == Status::Completed {
                    "Reopen"
                } else {
                    "Done"
                },
            ),
            Button::command(key.command("up")?, "Up").enabled(reorder && index > 0),
            Button::command(key.command("down")?, "Down")
                .enabled(reorder && index + 1 < model.order.len()),
            Button::command(key.command("delete")?, "Delete"),
        ],
    ))
}

fn presentation_diff(
    old_model: &TodoModel,
    old_session: TodoSession,
    new_model: &TodoModel,
    new_session: TodoSession,
) -> Result<Vec<PresentationOperation>> {
    let parent: NodeIdentity = node!("items").into();
    let old_children = child_identities(old_model, old_session)?;
    let new_children = child_identities(new_model, new_session)?;
    let mut operations = Vec::new();
    let mut staged = old_children.clone();

    for child in &old_children {
        if !new_children.contains(child) {
            operations.push(PresentationOperation::Remove {
                parent: parent.clone(),
                child: child.clone(),
            });
            staged.retain(|candidate| candidate != child);
        }
    }
    let retained = new_children
        .iter()
        .filter(|child| staged.contains(child))
        .cloned()
        .collect::<Vec<_>>();
    for (index, child) in retained.iter().enumerate() {
        let current = staged
            .iter()
            .position(|candidate| candidate == child)
            .expect("retained child remains staged");
        if current != index {
            operations.push(PresentationOperation::Move {
                parent: parent.clone(),
                child: child.clone(),
                index,
            });
            let child = staged.remove(current);
            staged.insert(index, child);
        }
    }
    for (index, child) in new_children.iter().enumerate() {
        if !staged.contains(child) {
            let subtree = child_element(child, new_model, new_session)?;
            operations.push(PresentationOperation::Insert {
                parent: parent.clone(),
                index,
                subtree,
            });
            staged.insert(index, child.clone());
        }
    }
    debug_assert_eq!(staged, new_children);

    property_diff(
        &mut operations,
        old_model,
        old_session,
        new_model,
        new_session,
    )?;
    Ok(operations)
}

fn child_identities(model: &TodoModel, session: TodoSession) -> Result<Vec<NodeIdentity>> {
    let ids = session.visible_ids(model);
    if ids.is_empty() {
        return Ok(vec![node!("empty").into()]);
    }
    ids.into_iter()
        .map(|id| Ok(item_key(id)?.node("row")?.into()))
        .collect()
}

fn child_element(
    identity: &NodeIdentity,
    model: &TodoModel,
    session: TodoSession,
) -> Result<Element> {
    if identity.id() == node!("empty").id() {
        return Ok(Text::new(node!("empty"), "No tasks"));
    }
    let item = session
        .visible_ids(model)
        .into_iter()
        .find(|id| {
            item_key(*id)
                .is_ok_and(|key| key.node("row").is_ok_and(|row| row.id() == identity.id()))
        })
        .and_then(|id| model.items.get(&id))
        .ok_or_else(|| Error::internal().with_message("presentation row identity is unknown"))?;
    row(item, model, session)
}

fn property_diff(
    operations: &mut Vec<PresentationOperation>,
    old_model: &TodoModel,
    old_session: TodoSession,
    new_model: &TodoModel,
    new_session: TodoSession,
) -> Result<()> {
    push_text(
        operations,
        node!("summary"),
        summary(old_model),
        summary(new_model),
    );
    push_text(
        operations,
        node!("page"),
        page_label(old_model, old_session),
        page_label(new_model, new_session),
    );
    for (key, old, new) in [
        (
            node!("filter-all"),
            old_session.filter != Filter::All,
            new_session.filter != Filter::All,
        ),
        (
            node!("filter-active"),
            old_session.filter != Filter::Active,
            new_session.filter != Filter::Active,
        ),
        (
            node!("filter-completed"),
            old_session.filter != Filter::Completed,
            new_session.filter != Filter::Completed,
        ),
        (node!("add"), can_add(old_model), can_add(new_model)),
        (
            node!("clear-completed"),
            has_completed(old_model),
            has_completed(new_model),
        ),
        (
            node!("previous"),
            old_session.page > 0,
            new_session.page > 0,
        ),
        (
            node!("next"),
            old_session.page + 1 < old_session.page_count(old_model),
            new_session.page + 1 < new_session.page_count(new_model),
        ),
    ] {
        if old != new {
            operations.push(PresentationOperation::Enabled(key.into(), new));
        }
    }

    for id in old_session
        .visible_ids(old_model)
        .into_iter()
        .filter(|id| new_session.visible_ids(new_model).contains(id))
    {
        let old_item = &old_model.items[&id];
        let new_item = &new_model.items[&id];
        let key = item_key(id)?;
        if old_item.status != new_item.status {
            operations.push(PresentationOperation::Text(
                key.node("status")?.into(),
                if new_item.status == Status::Completed {
                    "Completed".into()
                } else {
                    "Active".into()
                },
            ));
            operations.push(PresentationOperation::Label(
                key.node("toggle")?.into(),
                if new_item.status == Status::Completed {
                    "Reopen".into()
                } else {
                    "Done".into()
                },
            ));
        }
        let old_index = old_model
            .order
            .iter()
            .position(|candidate| *candidate == id)
            .unwrap();
        let new_index = new_model
            .order
            .iter()
            .position(|candidate| *candidate == id)
            .unwrap();
        let old_reorder = old_session.filter == Filter::All;
        let new_reorder = new_session.filter == Filter::All;
        for (role, old, new) in [
            (
                "up",
                old_reorder && old_index > 0,
                new_reorder && new_index > 0,
            ),
            (
                "down",
                old_reorder && old_index + 1 < old_model.order.len(),
                new_reorder && new_index + 1 < new_model.order.len(),
            ),
        ] {
            if old != new {
                operations.push(PresentationOperation::Enabled(key.node(role)?.into(), new));
            }
        }
    }
    Ok(())
}

fn push_text(operations: &mut Vec<PresentationOperation>, key: NodeKey, old: String, new: String) {
    if old != new {
        operations.push(PresentationOperation::Text(key.into(), new));
    }
}

fn into_update(operations: Vec<PresentationOperation>) -> Update {
    operations
        .into_iter()
        .fold(Update::new(), |update, operation| match operation {
            PresentationOperation::Remove { parent, child } => update.remove_subtree(parent, child),
            PresentationOperation::Move {
                parent,
                child,
                index,
            } => update.move_child(parent, child, index),
            PresentationOperation::Insert {
                parent,
                index,
                subtree,
            } => update.insert_child(parent, index, subtree),
            PresentationOperation::Text(key, value) => update.set_text(key, value),
            PresentationOperation::Label(key, value) => update.set_label(key, value),
            PresentationOperation::Enabled(key, value) => update.set_enabled(key, value),
        })
}

fn item_key(id: TaskId) -> Result<ItemKey> {
    ItemKey::new("todo", id.get())
}

fn summary(model: &TodoModel) -> String {
    format!(
        "{} active / {} total",
        model.active_count(),
        model.items.len()
    )
}

fn page_label(model: &TodoModel, session: TodoSession) -> String {
    format!("Page {} of {}", session.page + 1, session.page_count(model))
}

fn can_add(model: &TodoModel) -> bool {
    model.items.len() < MAX_ITEMS && model.next_id < u64::MAX
}

fn has_completed(model: &TodoModel) -> bool {
    model
        .items
        .values()
        .any(|item| item.status == Status::Completed)
}

fn app_error(error: ModelError) -> Error {
    match error {
        ModelError::Invalid(message) => Error::invalid_state().with_message(message),
        ModelError::Rejected(message) => Error::rejected_event().with_message(message),
    }
}

youth_sdk::export_app!(Todo);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_diff_orders_removal_movement_insertion_then_properties() {
        let mut old = TodoModel::default();
        let one = old.add().unwrap();
        let two = old.add().unwrap();
        let mut new = old.clone();
        new.delete(one).unwrap();
        let three = new.add().unwrap();
        new.move_by(three, -1).unwrap();
        let operations =
            presentation_diff(&old, TodoSession::default(), &new, TodoSession::default()).unwrap();
        let structural = operations
            .iter()
            .filter_map(|operation| match operation {
                PresentationOperation::Remove { .. } => Some("remove"),
                PresentationOperation::Move { .. } => Some("move"),
                PresentationOperation::Insert { .. } => Some("insert"),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(structural, vec!["remove", "insert"]);
        assert!(new.items.contains_key(&two));
    }

    #[test]
    fn filter_only_diff_has_no_model_mutation_and_replaces_projection() {
        let mut model = TodoModel::default();
        let one = model.add().unwrap();
        model.add().unwrap();
        model.toggle(one).unwrap();
        let old = TodoSession::default();
        let new = TodoSession {
            filter: Filter::Completed,
            page: 0,
        };
        let operations = presentation_diff(&model, old, &model, new).unwrap();
        assert!(
            operations
                .iter()
                .any(|operation| matches!(operation, PresentationOperation::Remove { .. }))
        );
        assert!(!operations.is_empty());
    }

    #[test]
    fn direct_reorder_authorization_does_not_trust_enabled_presentation() {
        let mut model = TodoModel::default();
        let first = model.add().unwrap();
        let mut session = TodoSession {
            filter: Filter::Active,
            page: 0,
        };
        assert_eq!(
            apply_action(&mut model, &mut session, CommandAction::MoveDown(first)),
            Err(ModelError::Rejected(
                "reordering is available only under All"
            ))
        );
    }
}
