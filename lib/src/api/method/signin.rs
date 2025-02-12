use crate::api::conn::Method;
use crate::api::conn::Param;
use crate::api::conn::Router;
use crate::api::Connection;
use crate::api::Result;
use crate::sql::Value;
use serde::de::DeserializeOwned;
use std::future::Future;
use std::future::IntoFuture;
use std::marker::PhantomData;
use std::pin::Pin;

/// A signin future
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Signin<'r, C: Connection, R> {
	pub(super) router: Result<&'r Router<C>>,
	pub(super) credentials: Result<Value>,
	pub(super) response_type: PhantomData<R>,
}

impl<'r, Client, R> IntoFuture for Signin<'r, Client, R>
where
	Client: Connection,
	R: DeserializeOwned + std::fmt::Debug,
{
	type Output = Result<R>;
	type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + Sync + 'r>>;

	fn into_future(self) -> Self::IntoFuture {
		let Signin {
			router,
			credentials,
			..
		} = self;
		Box::pin(async move {
			let router = router?;
			let mut conn = Client::new(Method::Signin);
			conn.execute(router, Param::new(vec![credentials?])).await
		})
	}
}
