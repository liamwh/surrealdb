use crate::api::err::Error;
use crate::api::Result;
use crate::sql;
use crate::sql::Array;
use crate::sql::Edges;
use crate::sql::Id;
use crate::sql::Object;
use crate::sql::Table;
use crate::sql::Thing;
use crate::sql::Value;
use std::ops;
use std::ops::Bound;

/// A database resource
#[derive(Debug)]
pub enum Resource {
	/// Table name
	Table(Table),
	/// Record ID
	RecordId(Thing),
	/// An object
	Object(Object),
	/// An array
	Array(Array),
	/// Edges
	Edges(Edges),
}

impl Resource {
	#[tracing::instrument(ret, err)]
	pub(crate) fn with_range(self, range: Range<Id>) -> Result<Value> {
		match self {
			Resource::Table(Table(table)) => Ok(sql::Range {
				tb: table,
				beg: range.start,
				end: range.end,
			}
			.into()),
			Resource::RecordId(record_id) => Err(Error::RangeOnRecordId(record_id).into()),
			Resource::Object(object) => Err(Error::RangeOnObject(object).into()),
			Resource::Array(array) => Err(Error::RangeOnArray(array).into()),
			Resource::Edges(edges) => Err(Error::RangeOnEdges(edges).into()),
		}
	}
}

impl From<Table> for Resource {
	fn from(table: Table) -> Self {
		Self::Table(table)
	}
}

impl From<Thing> for Resource {
	fn from(thing: Thing) -> Self {
		Self::RecordId(thing)
	}
}

impl From<Object> for Resource {
	fn from(object: Object) -> Self {
		Self::Object(object)
	}
}

impl From<Array> for Resource {
	fn from(array: Array) -> Self {
		Self::Array(array)
	}
}

impl From<Edges> for Resource {
	fn from(edges: Edges) -> Self {
		Self::Edges(edges)
	}
}

impl From<&str> for Resource {
	fn from(s: &str) -> Self {
		match sql::thing(s) {
			Ok(thing) => Self::RecordId(thing),
			Err(_) => Self::Table(s.into()),
		}
	}
}

impl From<&String> for Resource {
	fn from(s: &String) -> Self {
		Self::from(s.as_str())
	}
}

impl From<String> for Resource {
	fn from(s: String) -> Self {
		match sql::thing(s.as_str()) {
			Ok(thing) => Self::RecordId(thing),
			Err(_) => Self::Table(s.into()),
		}
	}
}

impl<T, I> From<(T, I)> for Resource
where
	T: Into<String>,
	I: Into<Id>,
{
	fn from((table, id): (T, I)) -> Self {
		let record_id = (table.into(), id.into());
		Self::RecordId(record_id.into())
	}
}

impl From<Resource> for Value {
	fn from(resource: Resource) -> Self {
		match resource {
			Resource::Table(resource) => resource.into(),
			Resource::RecordId(resource) => resource.into(),
			Resource::Object(resource) => resource.into(),
			Resource::Array(resource) => resource.into(),
			Resource::Edges(resource) => resource.into(),
		}
	}
}

/// A trait for converting inputs into database resources
pub trait IntoResource<Response>: Sized {
	/// Converts an input into a database resource
	fn into_resource(self) -> Result<Resource>;
}

impl IntoResource<Value> for Resource {
	#[tracing::instrument(ret, err)]
	fn into_resource(self) -> Result<Resource> {
		Ok(self)
	}
}

impl<R> IntoResource<Option<R>> for Object {
	#[tracing::instrument(ret, err)]
	fn into_resource(self) -> Result<Resource> {
		Ok(Resource::Object(self))
	}
}

impl<R> IntoResource<Option<R>> for Thing {
	#[tracing::instrument(ret, err)]
	fn into_resource(self) -> Result<Resource> {
		Ok(Resource::RecordId(self))
	}
}

impl<R> IntoResource<Option<R>> for &Thing {
	#[tracing::instrument(ret, err)]
	fn into_resource(self) -> Result<Resource> {
		Ok(Resource::RecordId(self.clone()))
	}
}

impl<R, T, I> IntoResource<Option<R>> for (T, I)
where
	T: Into<String> + std::fmt::Debug,
	I: Into<Id> + std::fmt::Debug,
{
	#[tracing::instrument(ret, err)]
	fn into_resource(self) -> Result<Resource> {
		let (table, id) = self;
		let record_id = (table.into(), id.into());
		Ok(Resource::RecordId(record_id.into()))
	}
}

impl<R> IntoResource<Vec<R>> for Array {
	#[tracing::instrument(ret, err)]
	fn into_resource(self) -> Result<Resource> {
		Ok(Resource::Array(self))
	}
}

impl<R> IntoResource<Vec<R>> for Edges {
	#[tracing::instrument(ret, err)]
	fn into_resource(self) -> Result<Resource> {
		Ok(Resource::Edges(self))
	}
}

impl<R> IntoResource<Vec<R>> for Table {
	#[tracing::instrument(ret, err)]
	fn into_resource(self) -> Result<Resource> {
		Ok(Resource::Table(self))
	}
}

fn blacklist_colon(input: &str) -> Result<()> {
	match input.contains(':') {
		true => {
			// We already know this string contains a colon
			let (table, id) = input.split_once(':').unwrap();
			Err(Error::TableColonId {
				table: table.to_owned(),
				id: id.to_owned(),
			}
			.into())
		}
		false => Ok(()),
	}
}

impl<R> IntoResource<Vec<R>> for &str {
	#[tracing::instrument(ret, err)]
	fn into_resource(self) -> Result<Resource> {
		blacklist_colon(self)?;
		Ok(Resource::Table(Table(self.to_owned())))
	}
}

impl<R> IntoResource<Vec<R>> for &String {
	#[tracing::instrument(ret, err)]
	fn into_resource(self) -> Result<Resource> {
		blacklist_colon(self)?;
		Ok(Resource::Table(Table(self.to_owned())))
	}
}

impl<R> IntoResource<Vec<R>> for String {
	#[tracing::instrument(ret, err)]
	fn into_resource(self) -> Result<Resource> {
		blacklist_colon(&self)?;
		Ok(Resource::Table(Table(self)))
	}
}

/// Holds the `start` and `end` bounds of a range query
#[derive(Debug)]
pub struct Range<T> {
	pub(crate) start: Bound<T>,
	pub(crate) end: Bound<T>,
}

impl<T> From<(Bound<T>, Bound<T>)> for Range<Id>
where
	T: Into<Id>,
{
	fn from((start, end): (Bound<T>, Bound<T>)) -> Self {
		Self {
			start: match start {
				Bound::Included(idx) => Bound::Included(idx.into()),
				Bound::Excluded(idx) => Bound::Excluded(idx.into()),
				Bound::Unbounded => Bound::Unbounded,
			},
			end: match end {
				Bound::Included(idx) => Bound::Included(idx.into()),
				Bound::Excluded(idx) => Bound::Excluded(idx.into()),
				Bound::Unbounded => Bound::Unbounded,
			},
		}
	}
}

impl<T> From<ops::Range<T>> for Range<Id>
where
	T: Into<Id>,
{
	fn from(
		ops::Range {
			start,
			end,
		}: ops::Range<T>,
	) -> Self {
		Self {
			start: Bound::Included(start.into()),
			end: Bound::Excluded(end.into()),
		}
	}
}

impl<T> From<ops::RangeInclusive<T>> for Range<Id>
where
	T: Into<Id>,
{
	fn from(range: ops::RangeInclusive<T>) -> Self {
		let (start, end) = range.into_inner();
		Self {
			start: Bound::Included(start.into()),
			end: Bound::Included(end.into()),
		}
	}
}

impl<T> From<ops::RangeFrom<T>> for Range<Id>
where
	T: Into<Id>,
{
	fn from(
		ops::RangeFrom {
			start,
		}: ops::RangeFrom<T>,
	) -> Self {
		Self {
			start: Bound::Included(start.into()),
			end: Bound::Unbounded,
		}
	}
}

impl<T> From<ops::RangeTo<T>> for Range<Id>
where
	T: Into<Id>,
{
	fn from(
		ops::RangeTo {
			end,
		}: ops::RangeTo<T>,
	) -> Self {
		Self {
			start: Bound::Unbounded,
			end: Bound::Excluded(end.into()),
		}
	}
}

impl<T> From<ops::RangeToInclusive<T>> for Range<Id>
where
	T: Into<Id>,
{
	fn from(
		ops::RangeToInclusive {
			end,
		}: ops::RangeToInclusive<T>,
	) -> Self {
		Self {
			start: Bound::Unbounded,
			end: Bound::Included(end.into()),
		}
	}
}

impl From<ops::RangeFull> for Range<Id> {
	fn from(_: ops::RangeFull) -> Self {
		Self {
			start: Bound::Unbounded,
			end: Bound::Unbounded,
		}
	}
}
