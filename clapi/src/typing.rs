use std::any::TypeId;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

/// Represents a type.
///
/// # Example
/// ```
/// use clapi::typing::Type;
/// use std::any::TypeId;
///
/// let r#type = Type::of::<i64>();
/// assert_eq!(r#type.name(), "i64");
/// assert_eq!(r#type.id(), TypeId::of::<i64>());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Type {
    type_name: &'static str,
    type_id: TypeId,
}

impl Type {
    /// Constructs a new `Type` from the given `T`.
    pub fn of<T: 'static>() -> Self {
    // pub const fn of<T: 'static>() -> Self {
        let type_name = std::any::type_name::<T>();
        let type_id = std::any::TypeId::of::<T>();
        Type { type_name, type_id }
    }

    /// Returns the type name of this type.
    pub const fn name(&self) -> &'static str {
        self.type_name
    }

    /// Returns the `TypeId` of this type.
    pub const fn id(&self) -> TypeId {
        self.type_id
    }
}

impl Eq for Type {}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

impl Ord for Type {
    fn cmp(&self, other: &Self) -> Ordering {
        self.type_id.cmp(&other.type_id)
    }
}

impl PartialOrd for Type {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.type_id.partial_cmp(&other.type_id)
    }
}

impl Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_id.hash(state)
    }
}