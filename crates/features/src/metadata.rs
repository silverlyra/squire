use std::{env, error::Error, fmt, ops::Deref};

use crate::{
    directive::{Directive, DirectiveKey, DirectiveValue, ParseDirectiveError, Threading},
    feature::{Configuration, FeatureKey, UnknownFeatureError},
    info::Library,
    probe::Probe,
    version::{ParseVersionError, Version},
};

#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(docsrs, doc(cfg(feature = "metadata")))]
pub struct Metadata {
    version: Version,
    features: Configuration,
    threading: Threading,
}

impl Metadata {
    pub const fn new(version: Version, features: Configuration, threading: Threading) -> Self {
        Self {
            version,
            features,
            threading,
        }
    }

    pub fn probe<P: Probe>(probe: &P) -> Result<Self, P::Error> {
        probe.probe().map(|library| Self::detect(&library))
    }

    pub fn detect(library: &Library) -> Self {
        let config = Configuration::detect(library);

        let threading = if let Some(Directive::Threading(threading)) =
            library.directive(DirectiveKey::Threading)
        {
            threading
        } else {
            Threading::default()
        };

        Self::new(library.version(), config, threading)
    }

    pub const fn version(&self) -> Version {
        self.version
    }

    pub const fn features(&self) -> &Configuration {
        &self.features
    }

    pub const fn threading(&self) -> Threading {
        self.threading
    }

    pub fn emit_for_dependents(&self) {
        println!("cargo:version={}", self.version);
        println!("cargo:threading={}", self.threading.value());

        let features: Vec<_> = self.enabled().map(|k| k.name()).collect();
        println!("cargo:features={}", features.join(","));
    }

    pub fn from_dependency() -> Result<Self, MetadataError> {
        let version = env::var("DEP_SQLITE3_VERSION")?.parse()?;
        let threading =
            Threading::parse_value(DirectiveKey::Threading, &env::var("DEP_SQLITE3_THREADING")?)?;
        let features = env::var("DEP_SQLITE3_FEATURES")?
            .split(',')
            .map(|key| key.parse())
            .collect::<Result<Configuration, _>>()?;

        Ok(Self::new(version, features, threading))
    }

    pub fn emit_cfg(&self) {
        for key in FeatureKey::all() {
            let name = key.cfg_name();
            println!("cargo::rustc-check-cfg=cfg(sqlite_has_{name})");
        }

        for key in self.enabled() {
            let name = key.cfg_name();
            println!("cargo::rustc-cfg=sqlite_has_{name}");
        }

        // Include SQLite library version
        println!("cargo::rustc-env=SQUIRE_SQLITE_VERSION={}", self.version());
    }
}

impl Deref for Metadata {
    type Target = Configuration;

    fn deref(&self) -> &Self::Target {
        &self.features
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "metadata")))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataError {
    Directive(ParseDirectiveError),
    Environment(env::VarError),
    Feature(UnknownFeatureError),
    Version(ParseVersionError),
}

impl fmt::Display for MetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Directive(err) => write!(f, "{err}"),
            Self::Environment(err) => write!(f, "{err}"),
            Self::Feature(err) => write!(f, "{err}"),
            Self::Version(err) => write!(f, "{err}"),
        }
    }
}

impl Error for MetadataError {}

impl From<ParseDirectiveError> for MetadataError {
    fn from(err: ParseDirectiveError) -> Self {
        MetadataError::Directive(err)
    }
}

impl From<env::VarError> for MetadataError {
    fn from(err: env::VarError) -> Self {
        MetadataError::Environment(err)
    }
}

impl From<UnknownFeatureError> for MetadataError {
    fn from(err: UnknownFeatureError) -> Self {
        MetadataError::Feature(err)
    }
}

impl From<ParseVersionError> for MetadataError {
    fn from(err: ParseVersionError) -> Self {
        MetadataError::Version(err)
    }
}
