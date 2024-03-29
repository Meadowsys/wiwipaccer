pub mod locale;
pub mod workspaces;

use crate::error::*;
use ::camino::Utf8PathBuf;
use ::std::ops::Deref;
use ::std::sync::Arc;
use ::surrealdb::Surreal;
use ::surrealdb::engine::local::{ Db, SpeeDb };
use ::surrealdb::opt::Config;
use ::surrealdb::dbs::Capabilities;
use ::tauri::{ AppHandle, Runtime };
#[cfg(not(debug_assertions))]
use ::tauri::Manager as _;
use ::tauri::State;
use ::tokio::sync::RwLock;
use ::tokio::sync::RwLockReadGuard;

const APPDATA_ROOTDIR: &str = ".wiwipaccer";
const APPDATA_DB_PATH: &str = "db";

const SETTINGS_TABLE: &str = "settings";

const NS: &str = "youare";
const DB: &str = "cute";

pub type DataTauriState<'h> = State<'h, AppDB>;

/// - [`Arc`]: allows owning reference to the same inner instance
/// - [`RwLock`]: allow write for creation purpose, but otherwise its only read
/// - [`Option`]: allow to take it out on app quit, so it can be dropped properly
pub struct AppDB {
	inner: Arc<RwLock<Option<Inner>>>
}

struct Inner {
	surreal: Surreal<Db>
}

impl AppDB {
	// param `handle` is (only) used in prod to get home dir
	#[cfg_attr(debug_assertions, allow(unused_variables))]
	pub async fn new<R: Runtime>(handle: &AppHandle<R>) -> Result<Self> {
		#[cfg(not(debug_assertions))]
		let appdata_path = handle.path().home_dir()
			.expect("couldn't get home dir")
			.to_str()
			.ok_or_else(|| Error::NonUtf8Path)?
			.to_string();

		#[cfg(debug_assertions)]
		let appdata_path = std::env::current_dir()
			.expect("couldn't get current dir")
			.to_str()
			.ok_or_else(|| Error::NonUtf8Path)?
			.to_string();

		let mut appdata_path = Utf8PathBuf::from(appdata_path);

		#[cfg(debug_assertions)]
		appdata_path.push("dev-home-dir");

		appdata_path.push(APPDATA_ROOTDIR);
		appdata_path.push(APPDATA_DB_PATH);

		let capabilities = Capabilities::all();
		let cfg = Config::default()
			.capabilities(capabilities);
		let surreal = Surreal::new::<SpeeDb>((appdata_path.as_str(), cfg))
			.await?;
		surreal.use_ns(NS).use_db(DB).await?;

		let inner = Arc::new(RwLock::new(Some(Inner { surreal })));
		Ok(Self { inner })
	}

	async fn surreal(&self) -> SurrealLock {
		SurrealLock::new(self.inner.read().await)
	}

	/// takes inner out, and drops it. **DO NOT** use self after calling this method,
	/// it will panic. Calling this multiple times will do nothing on subsequent
	/// calls. To be used _only_ when app is exiting, in the exit handler
	pub fn drop_db(&self) {
		let inner = self.inner.blocking_write().take();
		drop(inner);
	}
}

impl Clone for AppDB {
	#[inline]
	fn clone(&self) -> Self {
		let inner = Arc::clone(&self.inner);
		Self { inner }
	}
}

impl Drop for Inner {
	fn drop(&mut self) {
		println!("dropping db");
	}
}

#[repr(transparent)]
struct SurrealLock<'h> {
	lock: RwLockReadGuard<'h, Option<Inner>>
}

impl<'h> SurrealLock<'h> {
	fn new(lock: RwLockReadGuard<'h, Option<Inner>>) -> Self {
		Self { lock }
	}
}

impl<'h> Deref for SurrealLock<'h> {
	type Target = Surreal<Db>;

	#[inline]
	fn deref(&self) -> &Self::Target {
		match *self.lock {
			Some(ref inner) => { &inner.surreal }
			None => { unreachable!() }
		}
	}
}
