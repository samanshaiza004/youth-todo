use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const MAX_ITEMS: usize = 64;
pub const PAGE_SIZE: usize = 5;
pub const MAX_TITLE_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TaskId(u64);

impl TaskId {
    pub fn new(value: u64) -> Result<Self, ModelError> {
        if value == 0 {
            Err(ModelError::Invalid("task ID must be nonzero"))
        } else {
            Ok(Self(value))
        }
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    Active,
    Completed,
}

impl Status {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Completed => "completed",
        }
    }

    pub fn parse(value: &str) -> Result<Self, ModelError> {
        match value {
            "active" => Ok(Self::Active),
            "completed" => Ok(Self::Completed),
            _ => Err(ModelError::Invalid("item status is not canonical")),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TodoItem {
    pub id: TaskId,
    pub title: String,
    pub status: Status,
}

impl TodoItem {
    pub fn validate(&self) -> Result<(), ModelError> {
        if self.title.is_empty() {
            return Err(ModelError::Invalid("item title must not be empty"));
        }
        if self.title.len() > MAX_TITLE_BYTES {
            return Err(ModelError::Invalid("item title exceeds 256 UTF-8 bytes"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TodoModel {
    pub next_id: u64,
    pub order: Vec<TaskId>,
    pub items: BTreeMap<TaskId, TodoItem>,
}

impl Default for TodoModel {
    fn default() -> Self {
        Self {
            next_id: 1,
            order: Vec::new(),
            items: BTreeMap::new(),
        }
    }
}

impl TodoModel {
    pub fn validate(&self) -> Result<(), ModelError> {
        if self.next_id == 0 {
            return Err(ModelError::Invalid("next task ID must be nonzero"));
        }
        if self.order.len() > MAX_ITEMS || self.items.len() > MAX_ITEMS {
            return Err(ModelError::Invalid("collection exceeds 64 tasks"));
        }
        if self.order.len() != self.items.len() {
            return Err(ModelError::Invalid("order and item counts differ"));
        }
        let mut seen = BTreeSet::new();
        for id in &self.order {
            if !seen.insert(*id) {
                return Err(ModelError::Invalid("order contains a duplicate task ID"));
            }
            let item = self
                .items
                .get(id)
                .ok_or(ModelError::Invalid("order names a missing item"))?;
            if item.id != *id {
                return Err(ModelError::Invalid(
                    "item identity does not match its map key",
                ));
            }
            if id.get() >= self.next_id {
                return Err(ModelError::Invalid(
                    "next task ID must be greater than every item ID",
                ));
            }
            item.validate()?;
        }
        Ok(())
    }

    pub fn add(&mut self) -> Result<TaskId, ModelError> {
        self.validate()?;
        if self.items.len() == MAX_ITEMS {
            return Err(ModelError::Rejected("collection is full"));
        }
        if self.next_id == u64::MAX {
            return Err(ModelError::Rejected("task identity space is exhausted"));
        }
        let id = TaskId::new(self.next_id)?;
        self.next_id += 1;
        let item = TodoItem {
            id,
            title: format!("Task {}", id.get()),
            status: Status::Active,
        };
        item.validate()?;
        self.order.push(id);
        self.items.insert(id, item);
        Ok(id)
    }

    pub fn toggle(&mut self, id: TaskId) -> Result<(), ModelError> {
        let item = self
            .items
            .get_mut(&id)
            .ok_or(ModelError::Rejected("task does not exist"))?;
        item.status = match item.status {
            Status::Active => Status::Completed,
            Status::Completed => Status::Active,
        };
        Ok(())
    }

    pub fn delete(&mut self, id: TaskId) -> Result<(), ModelError> {
        if self.items.remove(&id).is_none() {
            return Err(ModelError::Rejected("task does not exist"));
        }
        self.order.retain(|candidate| *candidate != id);
        Ok(())
    }

    pub fn move_by(&mut self, id: TaskId, delta: isize) -> Result<(), ModelError> {
        let index = self
            .order
            .iter()
            .position(|candidate| *candidate == id)
            .ok_or(ModelError::Rejected("task does not exist"))?;
        let destination = index
            .checked_add_signed(delta)
            .filter(|destination| *destination < self.order.len())
            .ok_or(ModelError::Rejected("task cannot move farther"))?;
        self.order.swap(index, destination);
        Ok(())
    }

    pub fn clear_completed(&mut self) -> Result<Vec<TaskId>, ModelError> {
        let removed = self
            .order
            .iter()
            .copied()
            .filter(|id| self.items[id].status == Status::Completed)
            .collect::<Vec<_>>();
        if removed.is_empty() {
            return Err(ModelError::Rejected("no completed tasks exist"));
        }
        let removed_set = removed.iter().copied().collect::<BTreeSet<_>>();
        self.order.retain(|id| !removed_set.contains(id));
        for id in &removed {
            self.items.remove(id);
        }
        Ok(removed)
    }

    pub fn ordered(&self) -> impl ExactSizeIterator<Item = &TodoItem> {
        self.order.iter().map(|id| &self.items[id])
    }

    pub fn active_count(&self) -> usize {
        self.items
            .values()
            .filter(|item| item.status == Status::Active)
            .count()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Filter {
    #[default]
    All,
    Active,
    Completed,
}

impl Filter {
    pub const fn includes(self, status: Status) -> bool {
        match self {
            Self::All => true,
            Self::Active => matches!(status, Status::Active),
            Self::Completed => matches!(status, Status::Completed),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TodoSession {
    pub filter: Filter,
    pub page: usize,
}

impl TodoSession {
    pub fn filtered_ids(self, model: &TodoModel) -> Vec<TaskId> {
        model
            .ordered()
            .filter(|item| self.filter.includes(item.status))
            .map(|item| item.id)
            .collect()
    }

    pub fn page_count(self, model: &TodoModel) -> usize {
        self.filtered_ids(model).len().div_ceil(PAGE_SIZE).max(1)
    }

    pub fn clamp(&mut self, model: &TodoModel) {
        self.page = self.page.min(self.page_count(model) - 1);
    }

    pub fn visible_ids(self, model: &TodoModel) -> Vec<TaskId> {
        let filtered = self.filtered_ids(model);
        filtered
            .into_iter()
            .skip(self.page * PAGE_SIZE)
            .take(PAGE_SIZE)
            .collect()
    }

    pub fn set_filter(&mut self, filter: Filter) -> Result<(), ModelError> {
        if self.filter == filter {
            return Err(ModelError::Rejected("filter is already selected"));
        }
        self.filter = filter;
        self.page = 0;
        Ok(())
    }

    pub fn previous(&mut self) -> Result<(), ModelError> {
        self.page = self
            .page
            .checked_sub(1)
            .ok_or(ModelError::Rejected("already on the first page"))?;
        Ok(())
    }

    pub fn next(&mut self, model: &TodoModel) -> Result<(), ModelError> {
        if self.page + 1 >= self.page_count(model) {
            return Err(ModelError::Rejected("already on the last page"));
        }
        self.page += 1;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelError {
    Invalid(&'static str),
    Rejected(&'static str),
}

impl fmt::Display for ModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) | Self::Rejected(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for ModelError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_is_monotonic_and_never_reused() {
        let mut model = TodoModel::default();
        let first = model.add().unwrap();
        model.delete(first).unwrap();
        let second = model.add().unwrap();
        assert_eq!(first.get(), 1);
        assert_eq!(second.get(), 2);
        model.validate().unwrap();
    }

    #[test]
    fn collection_and_identity_boundaries_are_strict() {
        let mut model = TodoModel::default();
        for _ in 0..MAX_ITEMS {
            model.add().unwrap();
        }
        assert_eq!(model.add(), Err(ModelError::Rejected("collection is full")));
        let mut exhausted = TodoModel {
            next_id: u64::MAX,
            ..TodoModel::default()
        };
        assert!(matches!(exhausted.add(), Err(ModelError::Rejected(_))));
    }

    #[test]
    fn filtering_paging_and_clamping_use_the_filtered_projection() {
        let mut model = TodoModel::default();
        for _ in 0..7 {
            model.add().unwrap();
        }
        for id in [TaskId::new(1).unwrap(), TaskId::new(2).unwrap()] {
            model.toggle(id).unwrap();
        }
        let mut session = TodoSession::default();
        assert_eq!(session.page_count(&model), 2);
        session.set_filter(Filter::Completed).unwrap();
        assert_eq!(session.page_count(&model), 1);
        assert_eq!(session.visible_ids(&model).len(), 2);
        session.set_filter(Filter::Active).unwrap();
        assert_eq!(session.visible_ids(&model).len(), 5);
        model.delete(TaskId::new(7).unwrap()).unwrap();
        session.clamp(&model);
        assert_eq!(session.page, 0);
    }

    #[test]
    fn reorder_and_clear_completed_preserve_canonical_order() {
        let mut model = TodoModel::default();
        let one = model.add().unwrap();
        let two = model.add().unwrap();
        let three = model.add().unwrap();
        model.move_by(three, -1).unwrap();
        assert_eq!(model.order, vec![one, three, two]);
        model.toggle(three).unwrap();
        assert_eq!(model.clear_completed().unwrap(), vec![three]);
        assert_eq!(model.order, vec![one, two]);
        model.validate().unwrap();
    }
}
