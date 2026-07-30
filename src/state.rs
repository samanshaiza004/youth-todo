use std::collections::{BTreeMap, BTreeSet};

use youth_sdk::prelude::{Error, Result};
use youth_sdk::{StateReader, StateWriter};

use crate::model::{MAX_ITEMS, ModelError, Status, TaskId, TodoItem, TodoModel};

const SCHEMA_KEY: &str = "model-schema-version";
const NEXT_ID_KEY: &str = "todos-next-id";
const ORDER_KEY: &str = "todos-order";
const CURRENT_SCHEMA: i64 = 2;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedModel {
    pub model: TodoModel,
    pub needs_migration: bool,
}

pub trait StateRead {
    fn boolean(&self, key: &str) -> Result<Option<bool>>;
    fn integer(&self, key: &str) -> Result<Option<i64>>;
    fn text(&self, key: &str) -> Result<Option<String>>;
}

pub trait StateWrite: StateRead {
    fn set_integer(&mut self, key: &str, value: i64) -> Result<()>;
    fn set_text(&mut self, key: &str, value: &str) -> Result<()>;
    fn delete(&mut self, key: &str) -> Result<bool>;
}

impl StateRead for StateReader {
    fn boolean(&self, key: &str) -> Result<Option<bool>> {
        (*self).boolean(key)
    }

    fn integer(&self, key: &str) -> Result<Option<i64>> {
        (*self).integer(key)
    }

    fn text(&self, key: &str) -> Result<Option<String>> {
        (*self).text(key)
    }
}

impl StateRead for StateWriter {
    fn boolean(&self, key: &str) -> Result<Option<bool>> {
        (*self).boolean(key)
    }

    fn integer(&self, key: &str) -> Result<Option<i64>> {
        (*self).integer(key)
    }

    fn text(&self, key: &str) -> Result<Option<String>> {
        (*self).text(key)
    }
}

impl StateWrite for StateWriter {
    fn set_integer(&mut self, key: &str, value: i64) -> Result<()> {
        (*self).set_integer(key, value)
    }

    fn set_text(&mut self, key: &str, value: &str) -> Result<()> {
        (*self).set_text(key, value)
    }

    fn delete(&mut self, key: &str) -> Result<bool> {
        (*self).delete(key)
    }
}

pub fn load(reader: &impl StateRead) -> Result<LoadedModel> {
    let schema = reader.integer(SCHEMA_KEY)?;
    if schema.is_none() {
        if reader.text(NEXT_ID_KEY)?.is_some() || reader.text(ORDER_KEY)?.is_some() {
            return Err(invalid("schema marker is missing from a partial model"));
        }
        return Ok(LoadedModel {
            model: TodoModel::default(),
            needs_migration: false,
        });
    }
    let schema = schema.unwrap();
    if schema != 1 && schema != CURRENT_SCHEMA {
        return Err(invalid("model schema version is unsupported"));
    }
    let next_id = parse_canonical_u64(
        &reader
            .text(NEXT_ID_KEY)?
            .ok_or_else(|| invalid("next ID is missing"))?,
    )?;
    let order = parse_order(
        &reader
            .text(ORDER_KEY)?
            .ok_or_else(|| invalid("order is missing"))?,
    )?;
    let mut items = BTreeMap::new();
    for id in &order {
        let title = reader
            .text(&title_key(*id))?
            .ok_or_else(|| invalid("ordered item title is missing"))?;
        let status = if schema == 1 {
            if reader.text(&status_key(*id))?.is_some() {
                return Err(invalid("legacy item contains a current status"));
            }
            match reader
                .boolean(&done_key(*id))?
                .ok_or_else(|| invalid("legacy item completion flag is missing"))?
            {
                true => Status::Completed,
                false => Status::Active,
            }
        } else {
            if reader.boolean(&done_key(*id))?.is_some() {
                return Err(invalid("current item contains a legacy completion flag"));
            }
            let value = reader
                .text(&status_key(*id))?
                .ok_or_else(|| invalid("ordered item status is missing"))?;
            Status::parse(&value).map_err(model_invalid)?
        };
        let item = TodoItem {
            id: *id,
            title,
            status,
        };
        item.validate().map_err(model_invalid)?;
        items.insert(*id, item);
    }
    let model = TodoModel {
        next_id,
        order,
        items,
    };
    model.validate().map_err(model_invalid)?;
    Ok(LoadedModel {
        model,
        needs_migration: schema == 1,
    })
}

pub fn persist(
    writer: &mut impl StateWrite,
    old: &TodoModel,
    new: &TodoModel,
    needs_migration: bool,
) -> Result<()> {
    new.validate().map_err(model_invalid)?;
    writer.set_integer(SCHEMA_KEY, CURRENT_SCHEMA)?;
    writer.set_text(NEXT_ID_KEY, &new.next_id.to_string())?;
    writer.set_text(ORDER_KEY, &encode_order(&new.order))?;

    for id in &old.order {
        if !new.items.contains_key(id) {
            writer.delete(&title_key(*id))?;
            writer.delete(&status_key(*id))?;
            writer.delete(&done_key(*id))?;
        }
    }
    for item in new.ordered() {
        writer.set_text(&title_key(item.id), &item.title)?;
        writer.set_text(&status_key(item.id), item.status.as_str())?;
        if needs_migration {
            writer.delete(&done_key(item.id))?;
        }
    }
    Ok(())
}

pub fn encode_order(order: &[TaskId]) -> String {
    order
        .iter()
        .map(|id| id.get().to_string())
        .collect::<Vec<_>>()
        .join(",")
}

pub fn parse_order(value: &str) -> Result<Vec<TaskId>> {
    if value.is_empty() {
        return Ok(Vec::new());
    }
    let segments = value.split(',').collect::<Vec<_>>();
    if segments.len() > MAX_ITEMS {
        return Err(invalid("order exceeds 64 task IDs"));
    }
    let mut seen = BTreeSet::new();
    segments
        .into_iter()
        .map(|segment| {
            let id = TaskId::new(parse_canonical_u64(segment)?).map_err(model_invalid)?;
            if !seen.insert(id) {
                return Err(invalid("order contains a duplicate task ID"));
            }
            Ok(id)
        })
        .collect()
}

fn parse_canonical_u64(value: &str) -> Result<u64> {
    if value.is_empty()
        || value.starts_with('0') && value.len() > 1
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(invalid("unsigned integer text is not canonical"));
    }
    let parsed = value
        .parse::<u64>()
        .map_err(|_| invalid("unsigned integer text is out of range"))?;
    if parsed == 0 {
        return Err(invalid("unsigned integer text must be nonzero"));
    }
    Ok(parsed)
}

fn title_key(id: TaskId) -> String {
    format!("todo/{}/title", id.get())
}

fn status_key(id: TaskId) -> String {
    format!("todo/{}/status", id.get())
}

fn done_key(id: TaskId) -> String {
    format!("todo/{}/done", id.get())
}

fn invalid(message: &'static str) -> Error {
    Error::invalid_state().with_message(message)
}

fn model_invalid(error: ModelError) -> Error {
    Error::invalid_state().with_message(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Eq, PartialEq)]
    enum Value {
        Boolean(bool),
        Integer(i64),
        Text(String),
    }

    #[derive(Default)]
    struct FakeState(BTreeMap<String, Value>);

    impl StateRead for FakeState {
        fn boolean(&self, key: &str) -> Result<Option<bool>> {
            match self.0.get(key) {
                Some(Value::Boolean(value)) => Ok(Some(*value)),
                None => Ok(None),
                _ => Err(invalid("wrong fake value type")),
            }
        }

        fn integer(&self, key: &str) -> Result<Option<i64>> {
            match self.0.get(key) {
                Some(Value::Integer(value)) => Ok(Some(*value)),
                None => Ok(None),
                _ => Err(invalid("wrong fake value type")),
            }
        }

        fn text(&self, key: &str) -> Result<Option<String>> {
            match self.0.get(key) {
                Some(Value::Text(value)) => Ok(Some(value.clone())),
                None => Ok(None),
                _ => Err(invalid("wrong fake value type")),
            }
        }
    }

    impl StateWrite for FakeState {
        fn set_integer(&mut self, key: &str, value: i64) -> Result<()> {
            self.0.insert(key.into(), Value::Integer(value));
            Ok(())
        }

        fn set_text(&mut self, key: &str, value: &str) -> Result<()> {
            self.0.insert(key.into(), Value::Text(value.into()));
            Ok(())
        }

        fn delete(&mut self, key: &str) -> Result<bool> {
            Ok(self.0.remove(key).is_some())
        }
    }

    fn v1_state() -> FakeState {
        FakeState(BTreeMap::from([
            (SCHEMA_KEY.into(), Value::Integer(1)),
            (NEXT_ID_KEY.into(), Value::Text("3".into())),
            (ORDER_KEY.into(), Value::Text("1,2".into())),
            ("todo/1/title".into(), Value::Text("First".into())),
            ("todo/1/done".into(), Value::Boolean(false)),
            ("todo/2/title".into(), Value::Text("Second".into())),
            ("todo/2/done".into(), Value::Boolean(true)),
        ]))
    }

    #[test]
    fn canonical_order_round_trips_and_rejects_aliases() {
        let order = vec![TaskId::new(1).unwrap(), TaskId::new(42).unwrap()];
        assert_eq!(parse_order(&encode_order(&order)).unwrap(), order);
        assert_eq!(parse_order("").unwrap(), Vec::new());
        for invalid_value in ["0", "01", "+1", "1 2", "1,,2", "1,1", "1,"] {
            assert!(parse_order(invalid_value).is_err(), "{invalid_value:?}");
        }
    }

    #[test]
    fn absent_model_is_empty_but_partial_metadata_is_invalid() {
        assert_eq!(
            load(&FakeState::default()).unwrap().model,
            TodoModel::default()
        );
        let partial = FakeState(BTreeMap::from([(
            NEXT_ID_KEY.into(),
            Value::Text("1".into()),
        )]));
        assert!(load(&partial).is_err());
    }

    #[test]
    fn version_one_loads_read_only_and_migrates_atomically_as_operations() {
        let mut state = v1_state();
        let loaded = load(&state).unwrap();
        assert!(loaded.needs_migration);
        assert_eq!(
            loaded.model.items[&TaskId::new(2).unwrap()].status,
            Status::Completed
        );
        persist(&mut state, &loaded.model, &loaded.model, true).unwrap();
        let current = load(&state).unwrap();
        assert!(!current.needs_migration);
        assert!(!state.0.contains_key("todo/1/done"));
        assert_eq!(state.0["todo/2/status"], Value::Text("completed".into()));
    }

    #[test]
    fn strict_current_records_reject_legacy_or_partial_fields() {
        let mut state = v1_state();
        state.0.insert(SCHEMA_KEY.into(), Value::Integer(2));
        assert!(load(&state).is_err());
        state.0.remove("todo/1/done");
        state.0.remove("todo/2/done");
        state
            .0
            .insert("todo/1/status".into(), Value::Text("active".into()));
        assert!(load(&state).is_err());
    }
}
